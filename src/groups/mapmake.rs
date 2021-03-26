extern crate image;

use std::time::Instant;
use std::fs::File;

use serenity::prelude::*;
use serenity::model::{
    prelude::*,
};

use serenity::framework::standard::{
    Args, Delimiter, CommandResult,
    macros::command,
};

async fn get_map(m: ndarray::Array2<twmap::GameTile>) -> twmap::TwMap {
    let mut map = twmap::TwMap::empty(twmap::Version::DDNet06);

    let game = twmap::CompressedData::Loaded(m);

    map.groups.push(twmap::Group::game());
    map.groups[0].layers.push(twmap::Layer::Game(twmap::GameLayer { tiles: game }));

    map
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Map {
    tiles: ndarray::Array2<Tile>,
    width: usize,
    height: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Tile {
    pos: Point,
    tile: TileType,
    visited: bool
}

impl Tile {
    pub fn get_neighbors(self, map: &mut Map, filter: TileType, eight: bool) -> Vec<Tile> {
        let x = self.pos.x.clone();
        let y = self.pos.y.clone();

        let mut neighbors = vec![];

        if x >= 2 {
            if map.tiles[[x - 1, y]].tile == filter {
                neighbors.push(map.tiles[[x - 1, y]]);
            }

            if y >= 2 && map.tiles[[x - 1, y - 1]].tile == filter && eight {
                neighbors.push(map.tiles[[x - 1, y - 1]]);
            }
            if y <= map.height - 2 && map.tiles[[x - 1, y + 1]].tile == filter && eight {
                neighbors.push(map.tiles[[x - 1, y + 1]]);
            }

        }
        if x <= map.width - 2 {
            if map.tiles[[x + 1, y]].tile == filter {
                neighbors.push(map.tiles[[x + 1, y]]);
            }

            if y >= 2 && map.tiles[[x + 1, y - 1]].tile == filter && eight {
                neighbors.push(map.tiles[[x + 1, y - 1]]);
            }
            if y <= map.height - 2 && map.tiles[[x + 1, y + 1]].tile == filter && eight {
                neighbors.push(map.tiles[[x + 1, y + 1]]);
            }
        }

        if y >= 2 && map.tiles[[x, y - 1]].tile == filter {
            neighbors.push(map.tiles[[x, y - 1]]);
        }
        if y <= map.height - 2 && map.tiles[[x, y + 1]].tile == filter {
            neighbors.push(map.tiles[[x, y + 1]]);
        }

        neighbors
    }

    pub fn get_good_neighbors(self, map: &mut Map) -> Vec<Tile> {
        let neighbors = self.get_neighbors(map, TileType::Solid, false);

        let mut n: Vec<Tile> = vec![];

        for j in neighbors.iter() {
            let nx = j.pos.x as isize - self.pos.x as isize;
            let ny = j.pos.y as isize - self.pos.y as isize;

            for i in -1..1 as isize {
                let x = j.pos.x as isize + nx + i;
                let y = j.pos.y as isize + ny + i;

                if x < 2 || x >= map.width as isize - 2 || y < 2 || y >= map.width as isize - 2 {
                    continue;
                }

                if map.tiles[[x as usize, y as usize]].tile == TileType::Solid {
                    n.push(map.tiles[[j.pos.x, j.pos.y]]);
                }
            }
        }


        n
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Point {
    x: usize,
    y: usize
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TileType {
    Air,
    Solid,
    Unhookable
}

#[command("gen_map")]
pub async fn generate_map(ctx: &Context, msg: &Message, real_args: Args) -> CommandResult {
    let mut args = Args::new(real_args.message(), &[Delimiter::Single(' ')]);
    let width = args.single::<usize>()?;
    let height = args.single::<usize>()?;

    let start = Instant::now();

    // let mut n_msg = msg.channel_id.say(&ctx.http, "Generating map").await.unwrap();

    let mut map = map_algorithm(width, height).await;

    let _ = map.save_file("map.map");

    // let _ = n_msg.edit(&ctx, |m| m.content(format!("Map was converted to .map"))).await;

    let imgbuf = image::ImageBuffer::from_fn(width as u32, height as u32,
        |x, y| {
            let wtf: twmap::GameLayer = match &map.groups[0].layers[0] {
                twmap::Layer::Game(l) => l.clone(),
                _ => panic!("Got something wrong instead of Game layer.")
            };

            if wtf.tiles.unwrap()[[x as usize, y as usize]].id == 0 {
                image::Rgb([255, 255, 255])
            }
            else {
                image::Rgb([0, 0, 0]) 
            }
        });
    let _ = image::DynamicImage::ImageRgb16(imgbuf)
        .write_to(&mut File::create("map.png")?, image::ImageOutputFormat::Png)?;
    
    // let _ = n_msg.edit(&ctx, |m| m.content(format!("Map was converted to .png"))).await;



    let _ = msg.channel_id.send_files(&ctx.http, vec!["map.png", "map.map"], |m| {
        m.content(format!("Width: {}\nHeight: {}\n\nTime elapsed: {:?}", width, height, start.elapsed())) }).await;

    // let _ = n_msg.delete(&ctx.http).await;


    Ok(())
}

async fn map_algorithm(w: usize, h: usize) -> twmap::TwMap {
    let mut map = Map {
        tiles: vec![vec![Tile { pos: Point { x: 0, y: 0 }, tile: TileType::Solid, visited: false }; h]; w],
        width: w,
        height: h
    };

    for x in 0..w {
        for y in 0..h {
            map.tiles[[x, y]].pos.x = x;
            map.tiles[[x, y]].pos.y = y;
        }
    }

    let mut rng = urandom::new();

    let x = (rng.range(0..(map.width / 2)) * 2 + 1) as usize;
    let y = (rng.range(0..(map.height / 2)) * 2 + 1) as usize;    
    map.tiles[[x, y]].tile = TileType::Air;

    println!("{:?}", map.tiles[[x, y]]);

    let mut checkers = vec![map.tiles[[x, y]]];

    while !checkers.is_empty() {
        let i = rng.range(0..checkers.len() as usize);

        if map.tiles[[checkers[i].pos.x, checkers[i].pos.y]].tile == TileType::Air && map.tiles[[checkers[i].pos.x, checkers[i].pos.y]].visited == true {
            checkers.remove(i);
            continue;
        }

        let mut n = checkers[i].get_neighbors(&mut map, TileType::Solid, false);

        let d = checkers[i].get_good_neighbors(&mut map)
        .iter()
        .filter(|t| t.tile == TileType::Solid)
        .cloned()
        .collect::<Vec<Tile>>();

        if n.len() >= 3 && d.len() >= 3 && map.tiles[[checkers[i].pos.x, checkers[i].pos.y]].visited == false {
            map.tiles[[checkers[i].pos.x, checkers[i].pos.y]].tile = TileType::Air;

            while !n.is_empty() {
                let j = rng.range(0..n.len() as usize);
                if n[j].get_neighbors(&mut map, TileType::Solid, true).len() >= 5 && map.tiles[[n[j].pos.x, n[j].pos.y]].visited == false {
                    checkers.push(n[j]);
                }
                n.remove(j);
            }

            
        }

        map.tiles[[checkers[i].pos.x, checkers[i].pos.y]].visited = true;
        checkers.remove(i);
    }

    let u8_map: ndarray::Array2<twmap::GameTile> = ndarray::Array2::from_shape_fn((map.width, map.height),
    |(i, j)| { if map.tiles[[i, j]].tile == TileType::Solid { twmap::GameTile::new(1, twmap::TileFlags::empty()) } else { twmap::GameTile::new(0, twmap::TileFlags::empty()) }});

    get_map(u8_map).await

    // let mut scaled_map: Map = Map {
    //     tiles: vec![vec![Tile { pos: Point { x: 0, y: 0 }, tile: TileType::Solid, visited: false }; map.width * 4]; map.height * 4],
    //     width: map.width * 4,
    //     height: map.height * 4
    // };

    // for x in 0..map.width {
    //     for y in 0..map.height {
    //         for m in 0..4 {
    //             for n in 0..4 {
    //                 scaled_map.tiles[x * 4 + m][y * 4 + n].pos.x = x * 4 + m;
    //                 scaled_map.tiles[x * 4 + m][y * 4 + n].pos.y = y * 4 + n;
    //                 scaled_map.tiles[x * 4 + m][y * 4 + n].tile = map.tiles[x][y].tile;
    //                 scaled_map.tiles[x * 4 + m][y * 4 + n].visited = false;
    //             }
    //         }
    //     }
    // }

    // for x in 0..scaled_map.width {
    //     for y in 0..scaled_map.height {
    //         let n = scaled_map.tiles[x][y].get_neighbors(&mut scaled_map, false);
    //         let r = rng.range(0..n.len() as usize);
    //         if scaled_map.tiles[n[r].pos.x][n[r].pos.y].visited == false {
    //             scaled_map.tiles[n[r].pos.x][n[r].pos.y].tile = TileType::Air;
    //         }
    //     }
    // }

    // println!("{:?}", map.tiles[x][y]);
    
    // map
}