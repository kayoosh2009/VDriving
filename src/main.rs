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

    // 5. Текст для спидометра в левом верхнем углу
    commands.spawn((
        Text::new("Скорость: 0 км/ч"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            ..default()
        },
    ));
}

// Управление машиной
// Управление машиной и обновление спидометра
fn move_car(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut car_query: Query<(&mut Transform, &Car)>,
    mut text_query: Query<&mut Text>,
) {
    if let Ok((mut transform, car)) = car_query.get_single_mut() {
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

        let mut current_speed_kmh = 0.0;

        if direction != Vec3::ZERO {
            direction = direction.normalize();
            
            // Вычисляем смещение
            let movement = direction * car.speed * time.delta_secs();
            transform.translation += movement;
            
            // Примерный перевод абстрактной скорости Bevy в "километры в час" для интерфейса
            current_speed_kmh = car.speed * 4.0; 
            
            // Поворачиваем машину в сторону движения
            let target_rotation = Quat::from_rotation_y(direction.x.atan2(direction.z));
            transform.rotation = transform.rotation.lerp(target_rotation, 0.15);
        }

        // Обновляем текст спидометра
        if let Ok(mut text) = text_query.get_single_mut() {
            text.0 = format!("Скорость: {:.0} км/ч", current_speed_kmh);
        }
    }
}

// Система плавной слежки камеры за машиной
// Система плавной слежки камеры с динамическим сдвигом по X
fn camera_follow(
    car_query: Query<&Transform, (With<Car>, Without<MainCamera>)>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Car>)>,
    time: Res<Time>,
) {
    if let Ok(car_transform) = car_query.get_single() {
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            // Узнаем, куда "смотрит" машина (её направление вперед)
            let car_forward = car_transform.forward();
            
            // Камера пытается встать позади машины на основе её поворота, 
            // создавая крутой динамический занос камеры на поворотах!
            let target_camera_pos = car_transform.translation - car_forward * 12.0 + Vec3::new(0.0, 5.0, 0.0);
            
            // Плавно двигаем камеру к этой точке
            camera_transform.translation = camera_transform.translation.lerp(target_camera_pos, 3.0 * time.delta_secs());
            
            // Камера фокусируется чуть-чуть впереди машины, чтобы был виден горизонт
            let look_target = car_transform.translation + car_forward * 2.0;
            camera_transform.look_at(look_target, Vec3::Y);
        }
    }
}