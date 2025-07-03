# Design

## Systems

Stage and ordering of the systems is divided into `Set` (see `lib.rs:define_system_sets`):
1. InputSet
2. EditSet
3. ShowSet

Each tool and component should be registered as a plugin.