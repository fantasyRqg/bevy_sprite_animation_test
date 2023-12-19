use swj_utils::unit_team_system;

#[derive(Debug)]
struct TeamA;

#[derive(Debug)]
struct TeamB;

#[derive(Debug)]
struct TeamC;

fn system_a<T>() {
    println!("system_a: {:?}", std::any::type_name::<T>());
}

fn system_b<T>() {
    println!("system_b: {:?}", std::any::type_name::<T>());
}


fn main() {
    let systems = (system_a::<i32>,
                   unit_team_system!(TeamA, TeamB, TeamC;
                system_a,
                system_b,
        ),
                   system_b::<i32>,
    );

    systems.1.0();
}
