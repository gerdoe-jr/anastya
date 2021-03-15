extern crate image;

use rand::{thread_rng, Rng};
use std::fs::File;

use serenity::prelude::*;
use serenity::model::{
    prelude::*,
};

use serenity::framework::standard::{
    Args, Delimiter, CommandResult,
    macros::command,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Map {
    tiles: Vec<Vec<Tile>>,
}

impl Map {
    pub fn new(self, w: usize, h: usize, gen: fn(usize, usize)) -> Self {

    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Tile {
    tile: TileType
}

impl Tile {
    pub fn get_neighbors(self, cnt: Vec<Vec<Tile>>) -> Vec<Point> {
        vec![]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Point {
    x: usize,
    y: usize
}

impl Point {
    pub fn new(nx: usize, ny: usize) -> Self {
        Point{ x: nx, y: ny }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TileType {
    Air,
    Solid,
    Unhookable
}

enum Direction {
    Up,
    Down,
    Left,
    Right
}

#[command("gen_map")]
pub async fn generate_map(ctx: &Context, msg: &Message, real_args: Args) -> CommandResult {
    let mut args = Args::new(real_args.message(), &[Delimiter::Single(' ')]);
    let width = args.single::<usize>()?;
    let height = args.single::<usize>()?;
    let depth = args.single::<usize>()?;
    let iters = args.single::<usize>()?;

    let map = map_algorithm(width, height, depth).await;

    let imgbuf = image::ImageBuffer::from_fn(width as u32, height as u32,
        |x, y| {
            // println!("{:?}, {} {}", map[x as usize][y as usize], x, y);
            if map[x as usize][y as usize] == TileType::Air {
                image::Rgb([255, 255, 255])
            }
            else {
                image::Rgb([0, 0, 0]) 
            }
        });

    let output = image::DynamicImage::ImageRgb16(imgbuf)
        /*.thumbnail((width * 2) as u32, (height * 2) as u32)*/
        .write_to(&mut File::create("map.png")?, image::ImageOutputFormat::Png)?;

    let _ = msg.channel_id.send_files(&ctx.http, vec!["map.png"], |m| {
        m.content(format!("Width: {}\nHeight: {}\nDepth of penetration: {}", width, height, depth)) }).await;


    Ok(())
}

async fn map_algorithm(width: usize, height: usize, depth: usize) -> Vec<Vec<TileType>> {
    let mut map: Vec<Vec<TileType>> = vec![vec![TileType::Solid; width]; height];
    // println!("{:?}", map);

    let mut rng = thread_rng();
    let x = (rng.gen_range(0..(width / 2)) * 2 + 1) as usize;
    let y = (rng.gen_range(0..(height / 2)) * 2 + 1) as usize;
    map[x][y] = TileType::Air;

    println!("{:?}", map[x][y]);

    let mut checkers: Vec<Point> = Vec::new();

    if x >= 1 {
        checkers.push(Point::new(x - 1, y));
    }
    if x + 1 < width {
        checkers.push(Point::new(x + 1, y));
    }
    if y >= 1 {
        checkers.push(Point::new(x, y - 1));
    }
    if y + 1 < height {
        checkers.push(Point::new(x, y + 1));
    }

    while !checkers.is_empty() {
        let i = rng.gen_range(0..checkers.len()) as usize;
        map[checkers[i].x][checkers[i].y] = TileType::Air;

        let j = rng.gen_range(0..4) as usize;

        match j {
            0 => {
                if checkers[i].y >= 1 && map[checkers[i].x][checkers[i].y - 1] == TileType::Solid {
                    map[checkers[i].x][checkers[i].y - 1] = TileType::Air;
                }
            },
            1 => {
                if checkers[i].y + 1 < height && map[checkers[i].x][checkers[i].y + 1] == TileType::Solid {
                    map[checkers[i].x][checkers[i].y + 1] = TileType::Air;
                }
            },
            2 => {
                if checkers[i].x >= 1 && map[checkers[i].x - 1][checkers[i].y] == TileType::Solid {
                    map[checkers[i].x - 1][checkers[i].y] = TileType::Air;
                }
            },
            3 => {
                if checkers[i].x + 1 < width && map[checkers[i].x + 1][checkers[i].y] == TileType::Solid {
                    map[checkers[i].x + 1][checkers[i].y] = TileType::Air;
                }
            },
            _ => { }
        }

        // Add valid cells that are two orthogonal spaces away from the cell you cleared.
        if checkers[i].y >= 1 && map[checkers[i].x][checkers[i].y - 1] == TileType::Solid {
            checkers.push(Point::new(checkers[i].x, checkers[i].y - 1));
        }
        if checkers[i].y + 1 < height && map[checkers[i].x][checkers[i].y + 1] == TileType::Solid {
            checkers.push(Point::new(checkers[i].x, checkers[i].y + 1));
        }
        if checkers[i].x >= 1 && map[checkers[i].x - 1][checkers[i].y] == TileType::Solid {
            checkers.push(Point::new(checkers[i].x - 1, checkers[i].y));
        }
        if checkers[i].x + 1 < width && map[checkers[i].x + 1][checkers[i].y] == TileType::Solid {
            checkers.push(Point::new(checkers[i].x + 1, checkers[i].y));
        }

        checkers.remove(i);
        println!("{}", checkers.len());
    }

    for _ in 0..depth {
        let mut deadlock: Vec<Point> = Vec::new();

        for h in 0..height {
            for w in 0..width {
                if map[w][h] == TileType::Air {
                    let mut neighbors = 0;
                    if h >= 1 && map[w][h - 1] == TileType::Air {
                        neighbors += 1;
                    }
                    if h + 1 < height && map[w][h + 1] == TileType::Air {
                        neighbors += 1;
                    }
                    if w >= 1 && map[w - 1][h] == TileType::Air {
                        neighbors += 1;
                    }
                    if w + 1 < width && map[w + 1][h] == TileType::Air {
                        neighbors += 1;
                    }

                    if neighbors <= 1 {
                        deadlock.push(Point::new(w, h));
                    }
                }
            }
        }

        for d in deadlock.iter() {
            map[d.x][d.y] = TileType::Solid;
        }
    }

    /*for _ in 0..depth {
        // Add cell to the list if it has four or more cleared neighbors.
        let mut cleaning: Vec<Point> = Vec::new();
        for h in 0..height {
            for w in 0..width {
                if map[w][h] == TileType::Solid {
                    let mut neighbors = 0;

                    for a in 0..3 {
                        for b in 0..3 {
                            let mut nx = 0usize;
                            let mut ny = 0usize;

                            if w as i32 - a as i32 >= 0 {
                                nx = w - a;
                            }
                            if h as i32 - b as i32 >= 0 {
                                ny = h - b;
                            }

                            if nx < width && ny < height {
                                if map[nx][ny] == TileType::Air
                                 {
                                    neighbors += 1;
                                }
                            }
                        }
                    }

                    if neighbors >= 4 {
                        cleaning.push(Point::new(w, h));
                    }
                }
            }
        }
      
        for c in cleaning.iter() {
            map[c.x][c.y] = TileType::Air;
        }
    }*/

    println!("{:?}", map[x][y]);
    
    map
}