//! HOW TO CREATE ADVENTURE GAMES
//! by CHRISTOPHER LAMPTON (1986)
//!
//! This is a mostly faithful port of the adventure from BASIC.

use std::io;
use std::io::prelude::*;

/// These are just indices into the ROOMS array.
/// One-based. Zero is like None. Gross, I know!
type RoomId = u8;

/// Inventory is a special inaccessible room.
/// The book specifies -1 for this... but also likes using numbers larger than 127 (?)
const INVENTORY: RoomId = 255;

/// Line 20
const MAX_INVENTORY: usize = 5;

/// Static description of a room and its exits.
struct Room {
    desc: &'static str,
    n: RoomId, s: RoomId, e: RoomId, w: RoomId, u: RoomId, d: RoomId
}

/// Shortcut for rooms without vertical exits.
macro_rules! room {
    ($desc:expr, $n:expr, $s:expr, $e:expr, $w:expr) => {
        Room { desc: $desc, n: $n, s: $s, e: $e, w: $w, u: 0, d: 0 }
    }
}

/// Line 25000, 27000
static ROOMS: [Room; 20] = [
    room!("NOWHERE?", 0, 0, 0, 0),
    // house
    room!("IN YOUR LIVING ROOM.", 4, 3, 2, 0), // (1)
    room!("IN THE KITCHEN.",      0, 0, 0, 1),
    room!("IN THE LIBRARY.",      1, 0, 0, 0),
    room!("IN THE FRONT YARD.",   0, 1, 0, 5),
    room!("IN THE GARAGE.",       0, 0, 4, 0),
    // other world
    room!("IN AN OPEN FIELD.",             9, 7, 0, 0), // (6)
    room!("AT THE EDGE OF A FOREST.",      6, 0, 0, 0),
    Room { desc: "ON A BRANCH OF A TREE.", n:0, s:0, e:0, w:0, u:0, d:7 },
    room!("ON A LONG, WINDING ROAD.",      0, 6, 10, 0),
    room!("ON A LONG, WINDING ROAD.",      11, 0, 0, 9),
    room!("ON A LONG, WINDING ROAD.",      0, 10, 0, 12),
    room!("ON THE SOUTH BANK OF A RIVER.", 0, 0, 11, 0), // (12)
    room!("INSIDE THE WOODEN BOAT.",       0, 0, 0, 0),
    room!("ON THE NORTH BANK OF A RIVER.", 15, 0, 0, 0),
    room!("ON A WELL-TRAVELED ROAD.",      16, 14, 0, 0),
    room!("IN FRONT OF A LARGE CASTLE.",   GUARDED, 15, 0, 0), // (16)
    Room { desc: "IN A NARROW HALL.",      n:0, s:16, e:0, w:0, u:18, d:0  },
    Room { desc: "IN A LARGE HALL.",       n:0, s:0,  e:0, w:0, u:0,  d:17 },
    Room { desc: "ON THE TOP OF A TREE.",  n:0, s:0,  e:0, w:0, u:0,  d:8  },
];

const START_ROOM: RoomId = 1;
const KITCHEN: RoomId = 2;
const GARAGE: RoomId = 5;
const OPEN_FIELD: RoomId = 6;
const FOREST_EDGE: RoomId = 7;
const TREE_BRANCH: RoomId = 8;
const SOUTH_BANK: RoomId = 12;
const BOAT_ROOM: RoomId = 13;
const NORTH_BANK: RoomId = 14;
const NARROW_HALL: RoomId = 17;
const TREE_TOP: RoomId = 19;
const GUARDED: RoomId = 128;

/// Static description of an in-world object.
struct Object {
    name: &'static str,
    /// 3-letter noun prefix for lookup.
    tag: &'static str,
    /// Initial position.
    start: RoomId
}

macro_rules! immobile { ($n:expr) => ($n + 128) }

/// Indices into the OBJECTS array.
/// In rust this is far more practical as a usize, so we'll forgo the bytes.
type ObjId = usize;

/// Line 26000
const N_OBJECTS: ObjId = 18;
static OBJECTS: [Object; N_OBJECTS] = [
    Object { name: "DUMMY",                   tag: "DUM", start: 0 },
    Object { name: "AN OLD DIARY",            tag: "DIA", start: 1 },
    Object { name: "A SMALL BOX",             tag: "BOX", start: 1 },
    Object { name: "A CABINET",               tag: "CAB", start: immobile!(KITCHEN) },
    Object { name: "A SALT SHAKER",           tag: "SAL", start: 0 }, // (4)
    Object { name: "A DICTIONARY",            tag: "DIC", start: 3 },
    Object { name: "A WOODEN BARREL",         tag: "BAR", start: immobile!(GARAGE) },
    Object { name: "A SMALL BOTTLE",          tag: "BOT", start: 0 },
    Object { name: "A LADDER",                tag: "LAD", start: 4 }, // (8)
    Object { name: "A SHOVEL",                tag: "SHO", start: 5 },
    Object { name: "A TREE",                  tag: "TRE", start: immobile!(FOREST_EDGE) },
    Object { name: "A GOLDEN SWORD",          tag: "SWO", start: 0 },
    Object { name: "A WOODEN BOAT",           tag: "BOA", start: immobile!(SOUTH_BANK) },
    Object { name: "A MAGIC FAN",             tag: "FAN", start: 8 },
    Object { name: "A NASTY-LOOKING GUARD",   tag: "GUA", start: immobile!(16) },
    Object { name: "A GLASS CASE",            tag: "CAS", start: immobile!(18) },
    Object { name: "A GLOWING RUBY",          tag: "RUB", start: 0 }, // (16)
    Object { name: "A PAIR OF RUBBER GLOVES", tag: "GLO", start: 19 },
];

const SALT: ObjId = 4;
const BOTTLE: ObjId = 7;
const LADDER: ObjId = 8;
const SWORD: ObjId = 11;
const BOAT_OBJ: ObjId = 12;
const GUARD: ObjId = 14;
const RUBY: ObjId = 16;
const GLOVES: ObjId = 17;

struct State {
    /// Player position.
    here: RoomId,
    /// Position of each object in the game.
    positions: Vec<RoomId>,

    // various flags
    salted: bool,
    formulated: bool,
    gloved: bool,
    won: bool
}

impl State {
    fn new_game() -> Self {
        State {
            here: START_ROOM,
            positions: OBJECTS.iter().map(|obj| obj.start).collect(),
            salted: false, formulated: false, gloved: false,
            won: false
        }
    }

    fn look_around(&self) {
        let ref room = ROOMS[self.here as usize];
        // 700
        println!("\nYOU ARE {}", room.desc);
        // 500
        print!("YOU CAN GO:");
        // directions are put in an array at 25010
        if room.n > 0 { print!(" NORTH"); }
        if room.s > 0 { print!(" SOUTH"); }
        if room.e > 0 { print!(" EAST"); }
        if room.w > 0 { print!(" WEST"); }
        if room.d > 0 { print!(" DOWN"); }
        if room.u > 0 { print!(" UP"); }
        self.list_items_here();
    }

    fn list_items_here(&self) {
        // 600
        println!("\nYOU CAN SEE:");
        let mut found = false;
        for id in 0..N_OBJECTS {
            if self.positions[id] & 127 == self.here {
                indent();
                println!("{}", OBJECTS[id].name);
                found = true;
            }
        }
        if !found {
            indent();
            println!("THERE IS NOTHING OF INTEREST HERE.");
        }
    }

    /// Line 2000.
    fn go(&mut self, direction: Dir) {
        use Dir::*;
        let ref room = ROOMS[self.here as usize];
        let dest = match direction {
            N => room.n, S => room.s, E => room.e,
            W => room.w, U => room.u, D => room.d,
            Boat if self.here == SOUTH_BANK => BOAT_ROOM,
            Boat if self.here == NORTH_BANK => BOAT_ROOM,
            Boat => 0
        };
        if dest > 0 && dest < ROOMS.len() as u8 {
            self.here = dest;
            self.look_around();
        } else if dest == GUARDED { // Line 2410
            if self.positions[GUARD] != 0 {
                println!("THE GUARD WON'T LET YOU!");
            } else {
                self.here = NARROW_HALL;
                self.look_around();
            }
        } else {
            println!("YOU CAN'T GO THERE!");
        }
    }

    fn inventory(&self) {
        if self.gloved {
            println!("YOU ARE WEARING RUBBER GLOVES.");
        }
        println!("YOU ARE CARRYING:");
        let mut found = false;
        for id in 0..N_OBJECTS {
            if self.positions[id] == INVENTORY {
                indent();
                println!("{}", OBJECTS[id].name);
                found = true;
            }
        }
        if !found {
            indent();
            println!("NOTHING");
        }
    }

    fn inventory_count(&self) -> usize {
        self.positions.iter().filter(|&room| *room == INVENTORY).count()
    }

    /// Returns info about an object given the first 3 letters of its name.
    /// Line 1000
    fn lookup_object(&self, mut tag: &str) -> Option<(ObjId, RoomId)> {
        if tag == "SHA" { // Line 210
            tag = "SAL";
        }
        if tag == "FOR" { // Line 220
            tag = "BOT";
        }
        if let Some(id) = OBJECTS.iter().position(|obj| obj.tag == tag) {
            Some((id, self.positions[id]))
        } else {
            None
        }
    }

    /// Is the object in this room or in your inventory?
    fn is_object_here(&self, tag: &str) -> bool {
        if let Some((_, room)) = self.lookup_object(tag) {
            room == INVENTORY || room & 127 == self.here
        } else {
            false
        }
    }

    fn pick_up(&mut self, tag: &str) {
        if let Some((id, room)) = self.lookup_object(tag) { // Line 2520
            if room == INVENTORY { // 2530
                println!("YOU ALREADY HAVE IT!");
            } else if room > 127 { // 2540
                println!("YOU CAN'T GET THAT!");
            } else if room != self.here { // 2550
                not_found();
            } else if self.inventory_count() >= MAX_INVENTORY { // 2570
                println!("YOU CAN'T CARRY ANY MORE.");
            } else if tag == "RUB" { // 2575
                self.won = true;
            } else { // 2580
                self.positions[id] = INVENTORY;
                println!("TAKEN.");
            }
        } else {
            println!("YOU CAN'T GET THAT!");
        }
    }

    fn drop(&mut self, tag: &str) {
        if let Some((id, room)) = self.lookup_object(tag) {
            if room == INVENTORY {
                self.positions[id] = self.here;
                println!("DROPPED.");
                return; // early return!
            }
        }
        println!("YOU DON'T HAVE THAT!");
    }

    /// Line 2900
    fn examine(&self, tag: &str) {
        if tag == "GRO" { // 2910
            if self.here != OPEN_FIELD { // 2920
                println!("IT LOOKS LIKE GROUND!");
            } else if self.positions[SWORD] == 0 {
                println!("IT LOOKS LIKE SOMETHING'S BURIED HERE.");
            } else {
                println!("THERE'S A HOLE HERE.");
            }
        } else if self.is_object_here(tag) {
            println!("{}", match tag {
                "BOT" => "THERE'S SOMETHING WRITTEN ON IT!", // 3020
                "CAS" => "THERE'S A JEWEL INSIDE!", // 3030
                "BAR" => "IT'S FILLED WITH RAINWATER.",
                _ => "YOU SEE NOTHING UNUSUAL."
            });
        } else {
            not_found();
        }
    }

    fn open(&mut self, tag: &str) {
        if !self.is_object_here(tag) {
            not_found();
        } else if tag == "BOX" { // 3740
            // don't let the box infinitely respawn the bottle
            if self.positions[BOTTLE] == 0 {
                self.positions[BOTTLE] = self.here;
                println!("SOMETHING FELL OUT!");
                self.list_items_here();
            } else {
                println!("THE BOX IS ALREADY OPEN.");
            }
        } else if tag == "CAB" {
            if self.positions[SALT] == 0 {
                self.positions[SALT] = self.here;
                println!("THERE'S SOMETHING INSIDE!");
                self.list_items_here();
            } else {
                println!("THE CABINET IS ALREADY OPEN.");
            }
        } else if tag == "CAS" {
            if self.positions[RUBY] != 0 {
                println!("THE CASE IS ALREADY OPEN.");
            } else if self.gloved {
                println!("THE GLOVES INSULATE AGAINST THE");
                println!("ELECTRICITY! THE CASE OPENS!");
                self.positions[RUBY] = self.here;
                self.list_items_here();
            } else {
                println!("THE CASE IS ELECTRIFIED!");
            }
        } else {
            println!("YOU CAN'T OPEN THAT!");
        }
    }

    /// Line 3500
    fn read(&self, tag: &str) {
        if !self.is_object_here(tag) {
            not_found();
        } else if tag == "DIA" {
            println!("IT SAYS: 'ADD SODIUM CHLORIDE PLUS THE");
            println!("FORMULA TO RAINWATER, TO REACH THE");
            println!("OTHER WORLD.'");
        } else if tag == "DIC" {
            println!("IT SAYS: SODIUM CHLORIDE IS");
            println!("COMMON TABLE SALT.");
        } else if tag == "BOT" {
            println!("IT READS: 'SECRET FORMULA'.");
        } else {
            println!("YOU CAN'T READ THAT!");
        }
    }

    /// Line 3900
    fn pour(&mut self, tag: &str) {
        match tag {
            _ if !self.is_object_here(tag) => not_found(),
            "SAL" if self.salted => println!("THE SALT SHAKER IS EMPTY."),
            "SAL" if self.here == GARAGE => {
                self.salted = true;
                self.poured_into_barrel();
            }
            "BOT" if self.formulated => println!("THE BOTTLE IS EMPTY."),
            "BOT" if self.here == GARAGE => {
                self.formulated = true;
                self.poured_into_barrel();
            }
            _ => println!("YOU CAN'T POUR THAT!")
        }
    }

    fn poured_into_barrel(&mut self) {
        println!("POURED!");
        if self.salted && self.formulated { // Line 4010
            println!("THERE IS AN EXPLOSION!");
            println!("EVERYTHING GOES BLACK!");
            println!("SUDDENLY YOU ARE. . .");
            println!(". . .SOMEWHERE ELSE!");
            self.here = OPEN_FIELD;
            self.look_around();
        }
    }

    /// Line 4100
    fn climb(&mut self, tag: &str) {
        if tag == "TRE" && self.is_object_here("TRE") {
            println!("YOU CAN'T REACH THE BRANCHES!");
        } else if tag == "LAD" && self.is_object_here("LAD") {
            if self.here == FOREST_EDGE { // Line 4150
                println!("THE LADDER SINKS UNDER YOUR WEIGHT!");
                println!("IT DISAPPEARS INTO THE GROUND!");
                self.positions[LADDER] = 0;
            } else {
                println!("WHATEVER FOR?");
            }
        } else {
            println!("IT WON'T DO ANY GOOD.");
        }
    }

    /// Line 4300
    fn jump(&mut self) {
        if self.here == FOREST_EDGE {
            println!("YOU GRAB THE LOWEST BRANCH OF THE");
            println!("TREE AND PULL YOURSELF UP. . . .");
            self.here = TREE_BRANCH;
            self.look_around();
        } else if self.here == TREE_BRANCH {
            println!("YOU GRAB A HIGHER BRANCH OF THE");
            println!("TREE AND PULL YOURSELF UP. . . .");
            self.here = TREE_TOP;
            self.look_around();
        } else {
            println!("WHEE! THAT WAS FUN!");
        }
    }

    /// Line 4400
    fn dig(&mut self, obj: &str) {
        if obj != "GRO" && obj != "HOL" {
            println!("YOU CAN'T DIG THAT!");
        } else if !self.is_object_here("SHO") {
            println!("YOU DON'T HAVE A SHOVEL!");
        } else if self.here != OPEN_FIELD {
            println!("YOU DON'T FIND ANYTHING.");
        } else if self.positions[SWORD] != 0 {
            println!("THERE'S NOTHING ELSE THERE!");
        } else {
            println!("THERE'S SOMETHING THERE!");
            self.positions[SWORD] = OPEN_FIELD;
            self.list_items_here();
        }
    }

    /// Line 4500
    fn row_boat(&self) {
        if self.here != BOAT_ROOM {
            println!("YOU'RE NOT IN A BOAT!");
        } else {
            println!("YOU DON'T HAVE AN OAR!");
        }
    }

    /// Line 4600
    fn wave(&mut self, obj: &str) {
        if obj != "FAN" { // 4610
            println!("YOU CAN'T WAVE THAT!");
        } else if !self.is_object_here("FAN") { // 4615
            println!("YOU DON'T HAVE A FAN!");
        } else if self.here != BOAT_ROOM { // 4620
            println!("YOU FEEL A REFRESHING BREEZE!");
        } else { // 4630
            println!("A POWERFUL BREEZE PROPELS THE BOAT");
            println!("TO THE OPPOSITE SHORE!");
            if self.positions[BOAT_OBJ] == immobile!(SOUTH_BANK) {
                self.positions[BOAT_OBJ] = immobile!(NORTH_BANK);
            } else {
                self.positions[BOAT_OBJ] = immobile!(SOUTH_BANK);
            }
        }
    }

    /// Line 4700
    fn leave(&mut self, obj: &str) {
        if self.here == BOAT_ROOM {
            if obj == "BOA" {
                self.here = self.positions[BOAT_OBJ] & 127;
                self.look_around();
            } else {
                println!("HUH?"); // 4720
            }
        } else {
            println!("PLEASE GIVE A DIRECTION!"); // 4710
        }
    }

    /// Line 4800
    fn fight_guard(&mut self) {
        if !self.is_object_here("GUA") {
            println!("THERE'S NO GUARD HERE!");
        } else if self.positions[SWORD] != INVENTORY {
            println!("YOU DON'T HAVE A WEAPON!");
        } else {
            println!("THE GUARD, NOTICING YOUR SWORD,");
            println!("WISELY RETREATS INTO THE CASTLE.");
            self.positions[GUARD] = 0;
        }
    }

    fn wear_gloves(&mut self) {
        if self.gloved {
            println!("YOU ARE ALREADY WEARING THE RUBBER GLOVES.");
        } else if !self.is_object_here("GLO") {
            println!("YOU DON'T HAVE THE GLOVES.");
        } else {
            println!("YOU ARE NOW WEARING THE GLOVES.");
            self.gloved = true;
            self.positions[GLOVES] = 0;
        }
    }
}

fn not_found() {
    println!("THAT ISN'T HERE!");
}

fn indent() {
    print!("    ");
}

enum Dir { N, S, E, W, U, D, Boat }

impl Dir {
    fn parse(tag: &str) -> Option<Self> {
        use Dir::*;
        Some(match tag {
            "N" | "NOR" => N,
            "S" | "SOU" => S,
            "E" | "EAS" => E,
            "W" | "WES" => W,
            "U" | "UP"  => U,
            "D" | "DOW" => D,
            "BOA"       => Boat,
            _           => return None
        })
    }
}

/// Line 100
fn parser(input: &str, state: &mut State) -> bool {
    // Mr. Lampton specified to only use the first 3 letters of each word.
    let tags: Vec<&str> = input
        .split_ascii_whitespace()
        .map(|w| if w.len() > 3 { &w[..3] } else { w })
        .collect();
    if tags.is_empty() {
        return true;
    }
    // Are we going somewhere? ("GO" verb optional)
    let dir_tag = if tags.len() > 1 && tags[0] == "GO" { tags[1] } else { tags[0] };
    if let Some(dir) = Dir::parse(dir_tag) {
        state.go(dir);
        return true;
    }
    let miss = |verb| println!("WHAT DO YOU WANT TO {}?", verb);
    match AsRef::<[&str]>::as_ref(&tags) {
        ["Q"] | ["QUI"] => return false,
        ["I"] | ["INV"] => state.inventory(),
        ["L"] | ["LOO"] => state.look_around(),
        ["GO"] => println!("GO WHERE?"),

        ["EXA"] => miss("EXAMINE"),
        ["EXA", obj] | ["LOO", obj] => state.examine(obj),

        ["GET"] | ["TAK"] => miss("GET"),
        ["GET", item] | ["TAK", item] => state.pick_up(item),

        ["DRO"] => miss("DROP"),  ["DRO", item] => state.drop(item),
        ["OPE"] => miss("OPEN"),  ["OPE", obj] => state.open(obj),
        ["REA"] => miss("READ"),  ["REA", obj] => state.read(obj),
        ["POU"] => miss("POUR"),  ["POU", obj] => state.pour(obj),
        ["CLI"] => miss("CLIMB"), ["CLI", obj] => state.climb(obj),
        ["WAV"] => miss("WAVE"),  ["WAV", obj] => state.wave(obj),

        ["JUM"] | ["JUM", _] => state.jump(),
        ["DIG"] => state.dig("GRO"), ["DIG", obj] => state.dig(obj),
        ["ROW"] | ["ROW", "BOA"] => state.row_boat(),
        ["ROW", _] => println!("HOW CAN YOU ROW THAT?"),
        ["LEA"] | ["EXI"] => state.leave("BOA"),
        ["LEA", obj] | ["EXI", obj] => state.leave(obj),

        ["FIG"] => println!("WHOM DO YOU WANT TO FIGHT?"),
        ["FIG", "GUA"] => state.fight_guard(),
        ["FIG", _] => println!("YOU CAN'T FIGHT THEM!"),

        ["WEA"] => miss("WEA"),
        ["WEA", "GLO"] => state.wear_gloves(),
        ["WEA", _] => println!("YOU CAN'T WEAR THAT!"),

        _ => println!("I DON'T KNOW HOW TO DO THAT.")
    }
    true
}

fn intro() {
    println!(r#"
ALL YOUR LIFE YOU HAD HEARD THE STORIES
ABOUT YOUR CRAZY UNCLE SIMON. HE WAS AN
INVENTOR, WHO KEPT DISAPPEARING FOR
LONG PERIODS OF TIME, NEVER TELLING
ANYONE WHERE HE HAD BEEN.

YOU NEVER BELIEVED THE STORIES, BUT
WHEN YOUR UNCLE DIED AND LEFT YOU HIS
DIARY, YOU LEARNED THAT THEY WERE TRUE.
YOUR UNCLE HAD DISCOVERED A MAGIC
LAND, AND A SECRET FORMULA THAT COULD
TAKE HIM THERE. IN THAT LAND WAS A
MAGIC RUBY, AND HIS DIARY CONTAINED
THE INSTRUCTIONS FOR GOING THERE TO
FIND IT.
"#);
}

fn main() -> io::Result<()> {
    let mut state = State::new_game();
    intro();
    state.look_around();

    let mut input = String::new();
    while !state.won {
        input.clear();
        print!("\nWHAT NOW? ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        input.make_ascii_uppercase();
        if !parser(&input.trim(), &mut state) {
            break; // Line 3410, end game, skip confirmation
        }
    }

    if state.won { // Line 3430
        println!("\nCONGRATULATIONS! YOU'VE WON!\n");
        // skip replay question
    }

    Ok(())
}
