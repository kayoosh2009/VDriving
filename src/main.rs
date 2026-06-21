use bevy::prelude::*;
use bevy::image::{ImageSampler, ImageSamplerDescriptor};
use bevy::render::render_asset::RenderAssetUsages;

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
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    // 1. Свет (Солнце)
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 2. БОЛЬШАЯ КАРТА (Генерируем клетчатый пол)
    let mut texture_data = vec![0u8; 16 * 16 * 4];
    for y in 0..16 {
        for x in 0..16 {
            let idx = (y * 16 + x) * 4;
            let is_dark = (x / 2 + y / 2) % 2 == 0;
            let color = if is_dark { 50 } else { 100 };
            texture_data[idx] = color;     // R
            texture_data[idx + 1] = color; // G
            texture_data[idx + 2] = color; // B
            texture_data[idx + 3] = 255;   // A
        }
    }
    
    let mut texture = Image::new_fill(
        bevy::render::render_resource::Extent3d {
            width: 16,
            height: 16,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        &texture_data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    texture.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::linear());
    let texture_handle = images.add(texture);

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(1000.0, 1000.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle),
            // Задаем повторение текстуры через UV-координаты меша
            ..default()
        })),
    ));

    // 3. Машина (Загружаем Гольф из .glb через SceneRoot)
    commands.spawn((
        SceneRoot(asset_server.load("models/golf.glb#Scene0")), 
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(1.0)),
        Car { speed: 15.0 },
    ));

    // 4. Камера с маркером MainCamera
    commands.spawn((
        Camera3d::default(),
        MainCamera,
        Transform::from_xyz(0.0, 6.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 5. Текст для спидометра в ЛЕВОМ НИЖНЕМ углу
    commands.spawn((
        Text::new("Speed: 0 km/h"),
        TextFont {
            font_size: 35.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(30.0),
            left: Val::Px(30.0),
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
            text.0 = format!("Speed: {:.0} km/h", current_speed_kmh);
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