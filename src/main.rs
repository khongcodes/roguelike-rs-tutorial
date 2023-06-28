use rltk::{ Rltk, GameState, RGB, Point };
use specs::prelude::*;
use specs_derive::Component;

mod rect;
mod map;
mod player;
mod components;
mod visibility_system;
mod monster_ai_system;

use rect::Rect;
use map::{ Map, TileType, draw_map };
use player::{ Player, player_input };
use components::{ Position, Renderable, Viewshed, Monster, Name };
use visibility_system::VisiblitySystem;
use monster_ai_system::MonsterAI;


#[derive(PartialEq, Clone, Copy)]
pub enum RunState { Paused, Running }

pub struct State{
    pub ecs: World,
    pub runstate: RunState
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisiblitySystem{};
        vis.run_now(&self.ecs);
        
        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        // player_input(self, ctx);

        match self.runstate {
            RunState::Paused => {
                self.runstate = player_input(self, ctx);
            },
            RunState::Running => {
                self.run_systems();
                self.runstate = RunState::Paused;
            }
        }
        // self.run_systems();

        // This draws in visible and revealed walls and floors
        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        // creates a joined iterator - iterates only over entities with both
        // Position and Renderable components -
        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] { 
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph); 
            }
        }
    }
}


fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let context = RltkBuilder::simple80x50()
        .with_title("roguelike tutorial")
        .build()?;

    let mut gs = State {
        ecs: World::new(),
        runstate: RunState::Running
    };

    // Register new components with ECS, add to component storage
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();
    gs.ecs.insert(Point::new(player_x, player_y));

    // spawn monster in each room
    let mut rng = rltk::RandomNumberGenerator::new();
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x,y) = room.center();

        let glyph: rltk::FontCharType;
        let roll = rng.roll_dice(1, 2);
        let name: String;
        match roll {
            1 => { (glyph, name) = (rltk::to_cp437('g'), String::from("Goblin")) },
            _ => { (glyph, name) = (rltk::to_cp437('o'), String::from("Orc")) }
        }

        gs.ecs.create_entity()
            .with(Position {x, y})
            .with(Renderable{
                glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK)
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true
            })
            .with(Monster {})
            .with(Name { name: format!("{} #{}", &name, i) })
            .build();
    }
    gs.ecs.insert(map);

    gs.ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed { 
            visible_tiles: Vec::new(), 
            range: 8, 
            dirty: true 
        })
        .with(Name { name: String::from("Player") })
        .build();

    rltk::main_loop(context, gs)
}

