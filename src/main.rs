use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};

mod menu;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu, // Игра стартует в главном меню
    InGame,   // Сама гонка
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "VDriving - Vibe Cruise".into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>() // ✅ Теперь плагины загружаются первыми
        .add_plugins(menu::menu_plugin) // Подключаем плагин меню!
        // Системы игрового мира инициализируем только при ВХОДЕ в режим игры
        .add_systems(OnEnter(GameState::InGame), setup)
        // Управление и камеру обновляем ТОЛЬКО во время самой игры
        .add_systems(
            Update,
            (move_car, camera_follow, free_look_camera)
                .chain()
                .run_if(in_state(GameState::InGame)),
        )
        .run();
}

// Маркер для камеры, которую можно двигать вручную
#[derive(Component)]
struct FreeLookCamera;

// Состояние захвата мыши для вращения камеры
#[derive(Resource, Default)]
struct MouseGrabState {
    is_grabbed: bool,
}

#[derive(Component)]
struct Car {
    current_speed: f32, // Текущая скорость машины
    max_speed: f32,     // Максимальная скорость вперед
    acceleration: f32,  // Сила разгона
    deceleration: f32,  // Сила трения (когда бросаешь газ)
    rotation_speed: f32,// Скорость поворота руля
    angle: f32,         // Текущий угол направления машины
}

// Маркер для камеры, чтобы мы знали, какую именно двигать
#[derive(Component)]
struct MainCamera;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    let texture_handle = asset_server.load("floor.png");

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(1000.0, 1000.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle),
            // Важно: чтобы текстура повторялась, а не растягивалась
            ..default()
        })),
    ));

    let scale_factor = 1.0;

    // 3. Машина (Загружаем Гольф из .glb через SceneRoot)
    commands.spawn((
        SceneRoot(asset_server.load("models/Porsche_911_GT2.glb#Scene0")), // Укажи здесь точное имя твоего нового .glb файла
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(scale_factor)),
        Car {
            current_speed: 0.0,
            max_speed: 30.0,      // Максималка в Bevy-метрах
            acceleration: 15.0,   // Как быстро разгоняется
            deceleration: 8.0,    // Как быстро тормозит сама
            rotation_speed: 2.2,  // Насколько резко поворачивает руль
            angle: 0.0,           // Стартовый угол (смотрит вперед)
        },
    ));

    // 4. Камера с маркером MainCamera
    commands.spawn((
        Camera3d::default(),
        FreeLookCamera,
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

// Настоящее автомобильное управление и плавный разгон
fn move_car(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut car_query: Query<(&mut Transform, &mut Car)>,
    mut text_query: Query<&mut Text>,
) {
    if let Ok((mut transform, mut car)) = car_query.get_single_mut() {
        let dt = time.delta_secs();

        // 1. ПОВОРОТЫ (Управляем углом, только если машина движется)
        // Если едем назад, управление инвертируется (как в реальной жизни!)
        let speed_factor = (car.current_speed / car.max_speed).abs().clamp(0.0, 1.0);
        let direction_modifier = if car.current_speed >= 0.0 { 1.0 } else { -1.0 };

        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            car.angle += car.rotation_speed * speed_factor * direction_modifier * dt;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            car.angle -= car.rotation_speed * speed_factor * direction_modifier * dt;
        }

        // 2. РАЗГОН И ТОРМОЖЕНИЕ (W и S)
        let mut pressing_gas = false;

        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            car.current_speed += car.acceleration * dt;
            if car.current_speed > car.max_speed {
                car.current_speed = car.max_speed;
            }
            pressing_gas = true;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            car.current_speed -= car.acceleration * dt;
            if car.current_speed < -car.max_speed * 0.5 { // Назад едем в два раза медленнее максимума
                car.current_speed = -car.max_speed * 0.5;
            }
            pressing_gas = true;
        }

        // Плавное торможение (трение), когда мы отпускаем все педали
        if !pressing_gas {
            if car.current_speed > 0.0 {
                car.current_speed -= car.deceleration * dt;
                if car.current_speed < 0.0 { car.current_speed = 0.0; }
            } else if car.current_speed < 0.0 {
                car.current_speed += car.deceleration * dt;
                if car.current_speed > 0.0 { car.current_speed = 0.0; }
            }
        }

        // 3. ПЕРЕМЕЩЕНИЕ И ВРАЩЕНИЕ В ПРОСТРАНСТВЕ
        // Применяем текущий поворот к машине
        transform.rotation = Quat::from_rotation_y(car.angle);

        // Двигаем машину вперед относительно её собственного направления (ось -Z в Bevy)
        let forward = transform.forward();
        transform.translation += forward * car.current_speed * dt;

        // 4. ОБНОВЛЕНИЕ СПИДОМЕТРА (Переводим скорость в км/ч для красоты)
        let display_speed = (car.current_speed * 4.0).abs();
        if let Ok(mut text) = text_query.get_single_mut() {
            text.0 = format!("Speed: {:.0} km/h", display_speed);
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

fn free_look_camera(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<FreeLookCamera>>,
    mut grab_state: ResMut<MouseGrabState>,
) {
    if let Ok(mut cam_transform) = camera_query.get_single_mut() {
        // 1. Обработка зума (колесико)
        for event in scroll_events.read() {
            // Двигаем камеру вперед/назад по направлению взгляда
            let forward = cam_transform.forward().as_vec3();
            cam_transform.translation += forward * event.y * 5.0 * time.delta_secs();
        }

        // 2. Логика захвата мыши (ПКМ)
        if mouse_input.just_pressed(MouseButton::Right) {
            grab_state.is_grabbed = true;
        }
        if mouse_input.just_released(MouseButton::Right) {
            grab_state.is_grabbed = false;
        }

        // 3. Вращение камеры, если мышь захвачена
        if grab_state.is_grabbed {
            for event in mouse_motion.read() {
                let sensitivity = 0.005;
                
                // Вращение вокруг вертикальной оси (Y) - влево/вправо
                let yaw = Quat::from_rotation_y(-event.delta.x * sensitivity);
                
                // Вращение вокруг горизонтальной оси (X) - вверх/вниз
                // Ограничиваем угол, чтобы не перевернуться вверх ногами
                let pitch = Quat::from_rotation_x(-event.delta.y * sensitivity);
                
                cam_transform.rotation = yaw * cam_transform.rotation * pitch;
            }
        } else {
            // Очищаем события движения мыши, если она не захвачена, 
            // чтобы камера не дергалась при обычном движении курсора
            mouse_motion.clear();
        }
    }
}