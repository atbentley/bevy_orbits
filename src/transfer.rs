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

pub fn calculate_transfer(
    start_orbit: &Orbit,
    target_orbit: &Orbit,
    parent_mass: f32,
    execution_time: f32,
) -> Transfer {
    if start_orbit.eccentricity == 0.0 && target_orbit.eccentricity == 0.0 {
        return common_focus_circular_to_circular_hohmann_transfer(
            start_orbit,
            target_orbit,
            parent_mass,
            execution_time,
        );
    }

    common_focus_tangential_hohmann_transfer(start_orbit, target_orbit, parent_mass, execution_time)
}

pub fn common_focus_circular_to_circular_hohmann_transfer(
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

pub fn common_focus_tangential_hohmann_transfer(
    start_orbit: &Orbit,
    target_orbit: &Orbit,
    parent_mass: f32,
    execution_time: f32,
) -> Transfer {
    let time = execution_time;
    let start_period = calculate_period(start_orbit.semi_major_axis, parent_mass);
    let start_mean_motion = calculate_mean_motion(start_period);
    let departure_mean_anomaly = calculate_mean_anomaly(start_mean_motion, start_orbit.initial_mean_anomaly, time);
    let departure_eccentric_anomaly = calculate_eccentric_anomaly(start_orbit.eccentricity, departure_mean_anomaly);
    let departure_true_anomaly = if departure_mean_anomaly < PI {
        calculate_true_anomaly(start_orbit.eccentricity, departure_eccentric_anomaly)
    } else {
        TAU - calculate_true_anomaly(start_orbit.eccentricity, departure_eccentric_anomaly)
    };

    // a = acos((2 - k) k + 2 e.powi(2) - 2)/((k - 2) k)
    // b = acos((k + e^2 - 1)/(e k))
    // c = acos((1/e - e)/k - 1/e)

    let et = target_orbit.eccentricity;
    let f = |k: f32| -> f32 {
        println!("1 {}", (k + et.powi(2) - 1.0) / (et * k));
        println!("2 {}", ((k - et.powi(2) - 1.0) / (et * (k - 2.0))));
        println!("3 {}", (((2.0 - k) * k + 2.0 * et.powi(2) - 2.0) / ((k - 2.0) * k)));
        ((k + et.powi(2) - 1.0) / (et * k)).acos()
            + ((k - et.powi(2) - 1.0) / (et * (k - 2.0))).acos()
            + (((2.0 - k) * k + 2.0 * et.powi(2) - 2.0) / ((k - 2.0) * k)).acos()
            - PI
            - departure_true_anomaly
    };
    let df = |k: f32| -> f32 {
        (2.0 * (k - 1.0)
            * (-1.0
                * (1.0 - (-2.0 * et.powi(2) + k.powi(2) - 2.0 * k + 2.0).powi(2)).sqrt()
                * (2.0 * et.powi(2) - (k - 2.0) * k - 2.0).acos()
                + k.powi(2)
                - 2.0 * k))
            / ((k - 2.0).powi(2) * k.powi(2) * (1.0 - (-2.0 * et.powi(2) + k.powi(2) - 2.0 * k + 2.0).powi(2)).sqrt())
    };

    println!("START");
    let mut k = 1.0;
    for _i in 0..50 {
        println!("{},{},{}", k, f(k), df(k));

        k = (k - f(k) / df(k)).max(1.0 - et).min(1.0 + et);
    }
    // ((k + et.powi(2) - 1.0) / (et * k)).acos() + departure_true_anomaly - TAU
    let theta = if departure_true_anomaly > PI {
        -((k + et.powi(2) - 1.0) / (et * k)).acos() + departure_true_anomaly - TAU
    } else {
        departure_true_anomaly - ((k + et.powi(2) - 1.0) / (et * k)).acos()
    };
    let at = target_orbit.semi_major_axis;

    // TODO: in general this is orbital radius of the start orbit at the execution time
    let transfer_periapsis = start_orbit.semi_major_axis;
    let pt = transfer_periapsis;

    // TODO: divide by zero
    let transfer_semi_major_axis = pt * (at * k * theta.cos() + pt) / (at * k * (theta.cos() - 1.0) + 2.0 * pt);

    println!("theta {}", theta);
    println!("tsma {}", transfer_semi_major_axis);

    let transfer_eccentricity = 1.0 - transfer_periapsis / transfer_semi_major_axis;

    println!("dma {}", departure_mean_anomaly);
    println!("dta {}", departure_true_anomaly);
    // let transfer_argument_of_periapsis = if departure_true_anomaly < PI {
    //     TAU - departure_true_anomaly
    // } else {
    //     -departure_true_anomaly
    // };
    let transfer_argument_of_periapsis = -departure_true_anomaly;
    println!("taop {}", transfer_argument_of_periapsis);

    let transfer_orbit = Orbit {
        semi_major_axis: transfer_semi_major_axis,
        eccentricity: transfer_eccentricity,
        argument_of_periapsis: transfer_argument_of_periapsis,
        initial_mean_anomaly: 0.0,
    };

    let actual_target_orbit = Orbit {
        semi_major_axis: target_orbit.semi_major_axis,
        eccentricity: target_orbit.eccentricity,
        argument_of_periapsis: 0.0,
        initial_mean_anomaly: 0.0,
    };

    let maneuver_1 = Maneuver {
        start_orbit: start_orbit.clone(),
        target_orbit: transfer_orbit.clone(),
        execution_time: 0.0,
    };
    let maneuver_2 = Maneuver {
        start_orbit: transfer_orbit,
        target_orbit: actual_target_orbit,
        execution_time: 0.0,
    };

    Transfer {
        maneuvers: vec![maneuver_1, maneuver_2].into(),
    }
}

// pub fn common_focus_common_apse_line_transfer(
//     start_orbit: &Orbit,
//     target_orbit: &Orbit,
//     parent_mass: f32,
//     execution_time: f32,
// ) -> Transfer {
//     let target_semilatus_rectum = target_orbit.semi_major_axis * (1.0 - target_orbit.eccentricity.powi(2));

//     let start_period = calculate_period(start_orbit.semi_major_axis, parent_mass);
//     let start_mean_motion = calculate_mean_motion(start_period);
//     let departure_mean_anomaly =
//         calculate_mean_anomaly(start_mean_motion, start_orbit.initial_mean_anomaly, execution_time);
//     let departure_eccentric_anomaly = calculate_eccentric_anomaly(start_orbit.eccentricity, departure_mean_anomaly);
//     let departure_true_anomaly = calculate_true_anomaly(start_orbit.eccentricity, departure_eccentric_anomaly);
//     let departure_heliocentric_distance = calculate_heliocentric_distance(
//         start_orbit.semi_major_axis,
//         start_orbit.eccentricity,
//         departure_true_anomaly,
//     );

//     let arrival_true_anomaly = departure_true_anomaly + PI / 2.0;
//     let arrival_heliocentric_distance = calculate_heliocentric_distance(
//         target_orbit.semi_major_axis,
//         target_orbit.eccentricity,
//         arrival_true_anomaly,
//     );

//     let r_a = departure_heliocentric_distance;
//     let r_b = arrival_heliocentric_distance;
//     let theta_a = departure_true_anomaly;
//     let theta_b = arrival_true_anomaly;
//     let transfer_eccentricity = (r_b - r_a) / (r_a * theta_a.cos() - r_b * theta_b.cos());
//     let transfer_semi_latus_rectum =
//         r_a * r_b * (theta_a.cos() - theta_b.cos()) / (r_a * theta_a.cos() - r_b * theta_b.cos());
//     let transfer_semi_major_axis = transfer_semi_latus_rectum / (1.0 - transfer_eccentricity.powi(2));

//     let transfer_orbit = Orbit {
//         semi_major_axis: transfer_semi_major_axis,
//         eccentricity: transfer_eccentricity,
//         argument_of_periapsis: 0.0,
//         initial_mean_anomaly: 0.0,
//     };

//     let actual_target_orbit = Orbit {
//         semi_major_axis: target_orbit.semi_major_axis,
//         eccentricity: target_orbit.eccentricity,
//         argument_of_periapsis: 0.0,
//         initial_mean_anomaly: 0.0,
//     };

//     let maneuver_1 = Maneuver {
//         start_orbit: start_orbit.clone(),
//         target_orbit: transfer_orbit.clone(),
//         execution_time: 0.0,
//     };
//     let maneuver_2 = Maneuver {
//         start_orbit: transfer_orbit,
//         target_orbit: actual_target_orbit,
//         execution_time: 0.0,
//     };

//     Transfer {
//         maneuvers: vec![maneuver_1, maneuver_2].into(),
//     }
// }
