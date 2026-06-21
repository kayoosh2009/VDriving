use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "VDriving - Vibe Cruise".into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_car, camera_follow).chain()) // Камера обновляется ПОСЛЕ машины
        .run();
}

#[derive(Component)]
struct Car {
    speed: f32,
}

// Маркер для камеры, чтобы мы знали, какую именно двигать
#[derive(Component)]
struct MainCamera;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 1. Свет (Солнце)
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 2. БОЛЬШАЯ КАРТА (Пока просто огромная серая дорога-плоскость 500x500 метров)
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(500.0, 500.0))),
        MeshMaterial3d(materials.add(Color::rgb(0.2, 0.2, 0.2))), // Цвет асфальта
    ));

    // Добавим пару ярких кубов-декораций (как будто здания), чтобы видеть, что мы едем
    for z in [-20.0, -40.0, -60.0, 20.0, 40.0, 60.0] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(4.0, 10.0, 4.0))), // Высокое "здание"
            MeshMaterial3d(materials.add(Color::rgb(0.3, 0.4, 0.6))),
            Transform::from_xyz(15.0, 5.0, z),
        ));
    }

    // 3. Машина (наш красный куб)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.5, 1.0, 3.0))), // Сделали форму более вытянутой, как у тачки
        MeshMaterial3d(materials.add(Color::rgb(0.8, 0.1, 0.1))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Car { speed: 15.0 }, // Увеличили скорость для фана
    ));

    // 4. Камера с маркером MainCamera
    commands.spawn((
        Camera3d::default(),
        MainCamera,
        Transform::from_xyz(0.0, 6.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// Управление машиной
fn move_car(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Car)>,
) {
    if let Ok((mut transform, car)) = query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        if direction != Vec3::ZERO {
            direction = direction.normalize();
            transform.translation += direction * car.speed * time.delta_secs();
            
            // Поворачиваем "лицо" куба в сторону движения
            let target_rotation = Quat::from_rotation_y(direction.x.atan2(direction.z));
            transform.rotation = transform.rotation.lerp(target_rotation, 0.15);
        }
    }
}

// Система плавной слежки камеры за машиной
fn camera_follow(
    car_query: Query<&Transform, (With<Car>, Without<MainCamera>)>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Car>)>,
    time: Res<Time>,
) {
    if let Ok(car_transform) = car_query.get_single() {
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            // Камера должна быть сзади и сверху машины
            let target_camera_pos = car_transform.translation + Vec3::new(0.0, 5.0, 12.0);
            
            // Плавно передвигаем камеру к цели (интерполяция)
            camera_transform.translation = camera_transform.translation.lerp(target_camera_pos, 4.0 * time.delta_secs());
            
            // Камера всегда смотрит на машину
            camera_transform.look_at(car_transform.translation, Vec3::Y);
        }
    }
}