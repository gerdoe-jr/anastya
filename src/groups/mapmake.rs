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

#[derive(Clone, Debug)]
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

                if x < 2 || x >= (map.width as isize - 2) || y < 2 || y >= (map.height as isize - 2) {
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
    Air = 0,
    Solid = 1,
    Unhookable = 3
}

fn tile_to_gametile(tile: &TileType) -> twmap::GameTile {
    twmap::GameTile::new(*tile as usize, twmap::TileFlags::empty())
}

async fn resize_array(array: &ndarray::Array2<twmap::GameTile>, w: usize, h: usize, scale: usize) -> ndarray::Array2<twmap::GameTile> {
    ndarray::Array2::from_shape_fn((w * scale, h * scale), |(x, y)| { println!("x: {} -> {} y: {} -> {}", x, x / scale, y, y / scale); array[[x / scale, y / scale]] })
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

    println!("Map was converted to .map");

    let imgbuf = image::ImageBuffer::from_fn(3 * width as u32, 3 * height as u32,
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
    
        println!("Map was converted to .png");



    let _ = msg.channel_id.send_files(&ctx.http, vec!["map.png", "map.map"], |m| {
        m.content(format!("Width: {}\nHeight: {}\n\nTime elapsed: {:?}", width, height, start.elapsed())) }).await;

    // let _ = n_msg.delete(&ctx.http).await;


    Ok(())
}



async fn map_algorithm(w: usize, h: usize) -> twmap::TwMap {
    let mut map = Map {
        tiles: ndarray::Array2::from_shape_fn((w, h),
        |(i, j)| { Tile { tile: TileType::Solid, pos: Point { x: i, y: j }, visited: false}}),
        width: w,
        height: h
    };

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

        let d = checkers[i].get_good_neighbors(&mut map);

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

        println!("checkers: {}", checkers.len());
    }

    for _ in 0..4 {
        let deadlocks: Vec<Tile> = vec![];
        for x in 0..map.width {
            for y in 0..map.height {
                
            }
        }
    }

    let gt_map: ndarray::Array2<twmap::GameTile> = ndarray::Array2::from_shape_fn((map.width, map.height),
    |(i, j)| { tile_to_gametile(&map.tiles[[i, j]].tile) 
    });

    get_map(resize_array(&gt_map, map.width, map.height, 3).await).await
}