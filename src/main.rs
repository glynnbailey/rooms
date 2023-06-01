#![allow(dead_code)]

use rand::prelude::*;

const ROOMSIZE: usize = 10;
const FLOORSIZE: usize = 100;
const FLOORCOUNT: usize = 10;

struct Room {
    layout: Vec<Vec<TileType>>,
    connectors: Vec<Connector>,
    x: usize,
    y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TileType {
    None,
    Floor,
    Wall,
    Door,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Connector {
    x: usize,
    y: usize,
    direction: Direction
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

impl Room {
    fn new() -> Self {
        let mut rng = thread_rng();
        let mut room = Self {
            layout: vec![vec![TileType::None; ROOMSIZE]; ROOMSIZE],
            connectors: Vec::new(),
            x: 0,
            y: 0,
        };

        // generate dimensions
        let width = rng.gen_range(4..ROOMSIZE);
        let height = rng.gen_range(4..ROOMSIZE);

        // update layout
        for y in 1..height {
            for x in 1..width {
                room.layout[y][x] = TileType::Floor;
            }
        }

        // wall in and add connectors
        room.wall_in_floor();
        room.add_connectors_cross((width/2, height/2));
        
        room
    }

    fn wall_in_floor(&mut self) {
        for y in 0..ROOMSIZE as isize {
            for x in 0..ROOMSIZE as isize {
                if let TileType::Floor = self.layout[y as usize][x as usize] {

                    // turn any none tiles around the floor tile into wall
                    for y2 in -1..=1 {
                        for x2 in -1..=1 {
                            // panicing here due to going out of index bounds is acceptable as at this point floor tiles should always be
                            // surrounded by TileType::None so we can surround it in wall  
                            if let TileType::None = self.layout[(y+y2) as usize][(x+x2) as usize] {
                                self.layout[(y+y2) as usize][(x+x2) as usize] = TileType::Wall;
                            }
                        }
                    }

                }
            }
        }
    }

    fn add_connectors_cross(&mut self, (center_x, center_y): (usize, usize)) {
        // TODO check the entire rows in each direction rather than just the center and find the wall that sticks out the furthest to make connecting easier

        // north
        for y in 0..ROOMSIZE {
            if let TileType::Wall = self.layout[y][center_x] {
                self.connectors.push(Connector { x: center_x, y, direction: Direction::North });
                // self.layout[y][center_x] = TileType::Door; // door icon for debugging
                break
            }
        }

        // south
        for y in (0..ROOMSIZE).rev() {
            if let TileType::Wall = self.layout[y][center_x] {
                self.connectors.push(Connector { x: center_x, y, direction: Direction::South });
                // self.layout[y][center_x] = TileType::Door; // door icon for debugging
                break
            }
        }

        // west
        for x in 0..ROOMSIZE {
            if let TileType::Wall = self.layout[center_y][x] {
                self.connectors.push(Connector { x, y: center_y, direction: Direction::West });
                // self.layout[center_y][x] = TileType::Door; // door icon for debugging
                break
            }
        }

        // east
        for x in (0..ROOMSIZE).rev() {
            if let TileType::Wall = self.layout[center_y][x] {
                self.connectors.push(Connector { x, y: center_y, direction: Direction::East });
                // self.layout[center_y][x] = TileType::Door; // door icon for debugging
                break
            }
        }
    }

    fn print(&self) {
        for y in 0..ROOMSIZE {
            for x in 0..ROOMSIZE {
                match self.layout[y][x] {
                    TileType::None => print!("?"),
                    TileType::Floor => print!("."),
                    TileType::Wall => print!("#"),
                    TileType::Door => print!("+"),
                }
            }
            println!()
        }

        for connector in self.connectors.iter() {
            println!("{:?}", connector);
        }
    }
}

struct Map {
    layout: Vec<Vec<Vec<TileType>>>
}

impl Map {
    fn new() -> Self {
        Self {
            layout: vec![vec![vec![TileType::None; FLOORSIZE]; FLOORSIZE]; FLOORCOUNT]
        }
    }

    fn generate_floor(&mut self, z: usize, (start_x, start_y): (usize, usize)) {
        // sanity check the start coordinates to make sure they are on the map
        if start_x >= FLOORSIZE || start_y >= FLOORSIZE || z >= FLOORCOUNT {
            panic!("attempting to generate floor with start position larger than FLOORSIZE or zlevel larger than FLOORCOUNT")
        }

        let mut available_connectors = Vec::new();
        let mut blocked_connectors = Vec::new();

        // generate the first room
        let mut room = Room::new();

        // update the room so its center is at the start position, if it goes off the map set it to the map edge
        room.x = start_x - (ROOMSIZE/2);
        room.y = start_y - (ROOMSIZE/2);

        // update and store the connectors
        while let Some(mut connector) = room.connectors.pop() {
            connector.x += room.x;
            connector.y += room.y;
            available_connectors.push(connector);
        };

        // place the room on the map
        for y in 0..ROOMSIZE {
            for x in 0..ROOMSIZE {
                self.layout[z][room.y + y][room.x + x] = room.layout[y][x];
            }
        }

        // now loop through the available_connectors and try to add new rooms to them
        'available_connector_check: while let Some(available_connector) = available_connectors.pop() {
            
            // try multiple times to find a room that fits
            'new_room_check: for _ in 0..10 {
                let mut room = Room::new();
                'new_connector_check: for i in 0..room.connectors.len() {
                    if room.connectors[i].direction != available_connector.direction.opposite() {
                        continue;
                    }

                    let new_connector = room.connectors[i].clone();

                    // move the room into position and check it doesnt go out of bounds into negative space
                    room.x = match available_connector.x.checked_sub(new_connector.x) {
                        Some(x) => x,
                        None => continue 'new_connector_check,
                    };
                    room.y = match available_connector.y.checked_sub(new_connector.y) {
                        Some(y) => y,
                        None => continue 'new_connector_check,
                    };

                    // check the room doesnt go out of bounds into positive space
                    if room.x + ROOMSIZE >= FLOORSIZE || room.y + ROOMSIZE >= FLOORSIZE {
                        continue 'new_room_check
                    }

                    for y in 0..ROOMSIZE {
                        for x in 0..ROOMSIZE {
                            // check the room fits onto the map
                            match room.layout[y][x] {
                                // ignore None
                                TileType::None => continue,

                                // Floors can only go on None othewise it would pave over existing map features
                                TileType::Floor => match self.layout[z][room.y + y][room.x + x] {
                                    TileType::None => continue,
                                    _ => continue 'new_connector_check,
                                }

                                // Wall can only go on None or other Wall
                                TileType::Wall => match self.layout[z][room.y + y][room.x + x] {
                                    TileType::None => continue,
                                    TileType::Wall => continue,
                                    _ => continue 'new_connector_check,
                                },

                                // Door can only go onto None or Door
                                TileType::Door => match self.layout[z][room.y + y][room.x + x] {
                                    TileType::None => continue,
                                    TileType::Door => continue,
                                    _ => continue 'new_connector_check,
                                }
                            }
                        }
                    }

                    // the room fits, place it
                    for y in 0..ROOMSIZE {
                        for x in 0..ROOMSIZE {
                            if let TileType::None = room.layout[y][x] {
                                continue
                            }
                            self.layout[z][room.y + y][room.x + x] = room.layout[y][x];
                        }
                    }
                    self.layout[z][available_connector.y][available_connector.x] = TileType::Door;

                    // gather up the room's other connectors for use later
                    while let Some(mut connector) = room.connectors.pop() {
                        if connector != new_connector {
                            connector.x += room.x;
                            connector.y += room.y;
                            available_connectors.push(connector);
                        }
                    }
                    
                    // room has now placed and this connector is used, move on to the next available connector
                    continue 'available_connector_check
                }
            }

            // connector is blocked, store it for later to see if it can be used to join existing rooms
            blocked_connectors.push(available_connector);
        }

        // check blocked connectors to see if there are walls on both sides and if so turn it into a door
        for connector in blocked_connectors.iter() {
            // check map boundary
            if connector.x <= 0 || connector.x >= FLOORSIZE -1 || connector.y<= 0 || connector.y >= FLOORSIZE -1 {
                continue
            }

            if (self.layout[z][connector.y - 1][connector.x] == TileType::Floor && self.layout[z][connector.y + 1][connector.x] == TileType::Floor) ||
            (self.layout[z][connector.y][connector.x - 1] == TileType::Floor && self.layout[z][connector.y][connector.x + 1] == TileType::Floor)
            {
                self.layout[z][connector.y][connector.x] = TileType::Door;
            }
        }

        // turn all remaining None into Wall
        for y in 0..FLOORSIZE {
            for x in 0..FLOORSIZE {
                if let TileType::None = self.layout[z][y][x] {
                    self.layout[z][y][x] = TileType::Wall;
                }
            }
        }
    }

    fn print(&self, z: usize) {
        for y in 0..FLOORSIZE {
            for x in 0..FLOORSIZE {
                match self.layout[z][y][x] {
                    // TileType::None => panic!("TileType::None found in final map"),
                    TileType::None => print!("?"),
                    TileType::Floor => print!("."),
                    TileType::Wall => print!("#"),
                    TileType::Door => print!("+"),
                }
            }
            println!()
        }
    }
}




fn main() {
    // let room = Room::new();
    // room.print();

    let mut map = Map::new();
    map.generate_floor(0, (FLOORSIZE/2, FLOORSIZE/2));
    map.print(0)
}
