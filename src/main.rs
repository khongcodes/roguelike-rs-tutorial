use rltk::{ Rltk, GameState, RGB };
use specs::prelude::*;
use specs_derive::Component;

mod rect;
mod map;
mod player;
mod components;
mod visibility_system;

use rect::Rect;
use map::{ Map, TileType, draw_map };
use player::{ Player, player_input };
use components::{ Position, Renderable, Viewshed };
use visibility_system::VisiblitySystem;


pub struct State{
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisiblitySystem{};
        vis.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        player_input(self, ctx);
        self.run_systems();

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
        ecs: World::new()
    };

    // Register new components with ECS, add to component storage
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    // spawn monster in each room
    for room in map.rooms.iter().skip(1) {
        let (x,y) = room.center();
        gs.ecs.create_entity()
            .with(Position {x, y})
            .with(Renderable{
                glyph: rltk::to_cp437('g'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK)
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true
            })
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
        .build();

    rltk::main_loop(context, gs)
}

