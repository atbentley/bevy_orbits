use std::collections::VecDeque;
use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use crate::math::*;
use crate::orbit::Orbit;

#[derive(Debug, Clone)]
pub struct Maneuver {
    pub start_orbit: Orbit,
    pub target_orbit: Orbit,
    pub execution_time: f32,
}

#[derive(Debug, Default, Clone)]
pub struct Transfer {
    pub maneuvers: VecDeque<Maneuver>,
}

#[derive(Component, Default, Clone, Debug)]
pub struct TransferSchedule {
    pub transfers: VecDeque<Transfer>,
}

impl TransferSchedule {
    pub fn push_transfer(&mut self, transfer: Transfer) {
        self.transfers.push_back(transfer);
    }

    fn overdue_maneuver(&mut self, seconds: f32) -> Option<Maneuver> {
        let Some(next_transfer) = self.transfers.front_mut() else { return None };
        let Some(maybe_next_maneuver) = next_transfer.maneuvers.front() else { return None };

        if seconds < maybe_next_maneuver.execution_time {
            return None;
        };

        let next_maneuver = next_transfer.maneuvers.pop_front();

        if next_transfer.maneuvers.is_empty() {
            self.transfers.pop_front();
        }

        next_maneuver
    }
}

pub fn execute_orbital_maneuvers(time: Res<Time>, mut query: Query<(&mut Orbit, &mut TransferSchedule)>) {
    let seconds = time.elapsed_seconds();
    for (mut orbit, mut schedule) in query.iter_mut() {
        if let Some(next_maneuver) = schedule.overdue_maneuver(seconds) {
            orbit.semi_major_axis = next_maneuver.target_orbit.semi_major_axis;
            orbit.eccentricity = next_maneuver.target_orbit.eccentricity;
            orbit.argument_of_periapsis = next_maneuver.target_orbit.argument_of_periapsis;
            orbit.initial_mean_anomaly = next_maneuver.target_orbit.initial_mean_anomaly;
        }
    }
}

pub fn calculate_hohmann_transfer(
    start_orbit: &Orbit,
    target_orbit: &Orbit,
    parent_mass: f32,
    execution_time: f32,
) -> Transfer {
    let start_period = calculate_period(start_orbit.semi_major_axis, parent_mass);
    let start_mean_motion = calculate_mean_motion(start_period);
    let start_mean_anomaly = calculate_mean_anomaly(
        start_mean_motion,
        start_orbit.initial_mean_anomaly + start_orbit.argument_of_periapsis,
        execution_time,
    );

    let transfer_semi_major_axis = (start_orbit.semi_major_axis + target_orbit.semi_major_axis) / 2.0;
    let transfer_eccentricity = (1.0 - start_orbit.semi_major_axis / transfer_semi_major_axis).abs();
    let transfer_period = calculate_period(transfer_semi_major_axis, parent_mass);
    let transfer_argument_of_periapsis_offset = if start_orbit.semi_major_axis < transfer_semi_major_axis {
        0.0
    } else {
        PI
    };
    let transfer_argument_of_periapsis = -(start_mean_anomaly + transfer_argument_of_periapsis_offset).rem_euclid(TAU);
    let transfer_initial_mean_anomaly =
        calculate_initial_mean_anomaly(transfer_argument_of_periapsis_offset, transfer_period, execution_time);
    let transfer_orbit = Orbit {
        semi_major_axis: transfer_semi_major_axis,
        eccentricity: transfer_eccentricity,
        argument_of_periapsis: transfer_argument_of_periapsis,
        initial_mean_anomaly: transfer_initial_mean_anomaly,
    };

    let enter_transfer_orbit_time = execution_time;
    let exit_transfer_orbit_time = execution_time + transfer_period / 2.0;

    let target_period = calculate_period(target_orbit.semi_major_axis, parent_mass);
    let target_initial_mean_anomaly =
        calculate_initial_mean_anomaly(PI + start_mean_anomaly, target_period, exit_transfer_orbit_time);
    let actual_target_orbit = Orbit {
        semi_major_axis: target_orbit.semi_major_axis,
        eccentricity: target_orbit.eccentricity,
        argument_of_periapsis: 0.0,
        initial_mean_anomaly: target_initial_mean_anomaly,
    };

    let maneuver_1 = Maneuver {
        start_orbit: start_orbit.clone(),
        target_orbit: transfer_orbit.clone(),
        execution_time: enter_transfer_orbit_time,
    };
    let maneuver_2 = Maneuver {
        start_orbit: transfer_orbit,
        target_orbit: actual_target_orbit,
        execution_time: exit_transfer_orbit_time,
    };

    Transfer {
        maneuvers: vec![maneuver_1, maneuver_2].into(),
    }
}
