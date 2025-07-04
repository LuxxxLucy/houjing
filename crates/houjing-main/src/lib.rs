mod component;
mod systems;

use bevy::prelude::*;

macro_rules! define_system_sets {
    ($($name:ident),*) => {
        $(
            #[derive(bevy::ecs::schedule::SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
            pub struct $name;
        )*
    };
}

// we define the system sets here as different stages of the processing pipeline.
define_system_sets!(
    InputSet, // handle input
    EditSet,  // edit
    ShowSet   // visualize
);

fn init_app(app: &mut App) {
    component::add_component_plugins(app);
    systems::add_systems_plugins(app);
}

// Extension trait for App to provide builder pattern used in `main.rs`
pub trait Application {
    fn init(self) -> Self;
}

impl Application for App {
    fn init(mut self) -> Self {
        init_app(&mut self);
        self
    }
}
