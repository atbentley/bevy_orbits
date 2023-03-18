pub mod math;
mod orbit;
mod plugin;
mod transfer;

pub mod prelude {
    pub use crate::orbit::{Mass, Orbit};
    pub use crate::plugin::OrbitPlugin;
    pub use crate::transfer::{calculate_hohmann_transfer, Maneuver, Transfer, TransferSchedule};
}
