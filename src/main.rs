use rand::prelude::*;

const ROOMSIZE: usize = 10;
const FLOORSIZE: usize = 40;

struct Room {
    layout: Vec<Vec<TileType>>,
    connectors: Vec<Connector>,
    center_x: usize,
    center_y: usize,
}

#[derive(Debug, Clone, Copy)]
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
            center_x: 0,
            center_y: 0,
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

        // record room center
        room.center_x = width/2;
        room.center_y = height/2;

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
                self.layout[y][center_x] = TileType::Door; // door icon for debugging
                break
            }
        }

        // south
        for y in (0..ROOMSIZE).rev() {
            if let TileType::Wall = self.layout[y][center_x] {
                self.connectors.push(Connector { x: center_x, y, direction: Direction::South });
                self.layout[y][center_x] = TileType::Door; // door icon for debugging
                break
            }
        }

        // west
        for x in 0..ROOMSIZE {
            if let TileType::Wall = self.layout[center_y][x] {
                self.connectors.push(Connector { x, y: center_y, direction: Direction::West });
                self.layout[center_y][x] = TileType::Door; // door icon for debugging
                break
            }
        }

        // east
        for x in (0..ROOMSIZE).rev() {
            if let TileType::Wall = self.layout[center_y][x] {
                self.connectors.push(Connector { x, y: center_y, direction: Direction::East });
                self.layout[center_y][x] = TileType::Door; // door icon for debugging
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
    layout: Vec<Vec<TileType>>
}

impl Map {
    fn new((start_x, start_y): (usize, usize)) -> Self {
        let mut map = Self {
            layout: vec![vec![TileType::None; FLOORSIZE]; FLOORSIZE]
        };
        let mut available_connectors = Vec::new();
        let mut blocked_connectors = Vec::new();

        // generate the first room
        let mut room = Room::new();

        // update and store the connectors
        while let Some(mut connector) = room.connectors.pop() {
                connector.x += start_x;
                connector.y += start_y;
                available_connectors.push(connector);
        };

        // place the room with the center at the start position
        let offset_x = start_x - room.center_x;
        let offset_y = start_y - room.center_y;
        for room_y in 0..ROOMSIZE {
            for room_x in 0..ROOMSIZE {
                map.layout[offset_y + room_y][offset_x + room_x] = room.layout[room_y][room_x];
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

                    // move the room into position and check if it will fit
                    let (offset_x, overflow_x) = available_connector.x.overflowing_sub(new_connector.x);
                    let (offset_y, overflow_y) = available_connector.y.overflowing_sub(new_connector.y);

                    // check the room doesnt go out of bounds
                    if overflow_x || overflow_y || offset_x + ROOMSIZE >= FLOORSIZE || offset_y + ROOMSIZE >= FLOORSIZE {
                        continue 'new_room_check
                    }

                    for room_y in 0..ROOMSIZE {
                        for room_x in 0..ROOMSIZE {
                            // check the room fits onto the map
                            match room.layout[room_y][room_x] {
                                // ignore None
                                TileType::None => continue,

                                // floors can only go on None othewise it would pave over existing map features
                                TileType::Floor => match map.layout[offset_y + room_y][offset_x + room_x] {
                                    TileType::None => continue,
                                    _ => continue 'new_connector_check,
                                }

                                // walls can only go on None or other Wall
                                TileType::Wall => match map.layout[offset_y + room_y][offset_x + room_x] {
                                    TileType::None => continue,
                                    TileType::Wall => continue,
                                    _ => continue 'new_connector_check,
                                },

                                // doors can only go onto None or Door
                                TileType::Door => match map.layout[offset_y + room_y][offset_x + room_x] {
                                    TileType::None => continue,
                                    TileType::Door => continue,
                                    _ => continue 'new_connector_check,
                                }
                            }
                        }
                    }

                    println!("fits");

                    // the room fits, place it
                    for room_y in 0..ROOMSIZE {
                        for room_x in 0..ROOMSIZE {
                            map.layout[offset_y + room_y][offset_x + room_x] = room.layout[room_y][room_x];
                        }
                    }

                    // gather up the new rooms other connectors for use later
                    while let Some(connector) = room.connectors.pop() {
                        if connector != new_connector {
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

        // check blocked connectors

        map
    }

    fn print(&self) {
        for y in 0..FLOORSIZE {
            for x in 0..FLOORSIZE {
                match self.layout[y][x] {
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

    let map = Map::new((FLOORSIZE/2, FLOORSIZE/2));
    map.print()
}
