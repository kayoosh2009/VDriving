use bevy::prelude::*;
use crate::GameState;

// Маркер для сущностей меню, чтобы мы могли удалить их все разом при старте игры
#[derive(Component)]
pub struct OnMainMenuScreen;

pub fn menu_plugin(app: &mut App) {
    app
        // Когда входим в состояние MainMenu — создаем интерфейс
        .add_systems(OnEnter(GameState::MainMenu), spawn_menu)
        // Когда выходим из MainMenu — удаляем интерфейс (деспавним)
        .add_systems(OnExit(GameState::MainMenu), cleanup_menu)
        // Во время нахождения в меню — проверяем нажатия на кнопки
        .add_systems(Update, button_system.run_if(in_state(GameState::MainMenu)));
}

// Создание интерфейса меню
fn spawn_menu(mut commands: Commands) {
    // Камера для UI (без неё мы не увидим 2D текст и кнопки)
    commands.spawn((Camera2d::default(), OnMainMenuScreen));

    // Корневой контейнер на весь экран (Фон меню)
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::BLACK), // Пока сделаем стильный черный фон
            OnMainMenuScreen,
        ))
        .with_children(|parent| {
            // ЛОГОТИП / НАЗВАНИЕ ИГРЫ
            parent.spawn((
                Text::new("VDriving"),
                TextFont {
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ));

            // КНОПКА "PLAY"
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(Color::WHITE),
                    BackgroundColor(Color::rgb(0.15, 0.15, 0.15)),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("PLAY"),
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

// Логика взаимодействия с кнопкой
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut background_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // При нажатии переключаем состояние на InGame!
                next_state.set(GameState::InGame);
            }
            Interaction::Hovered => {
                *background_color = BackgroundColor(Color::rgb(0.25, 0.25, 0.25));
            }
            Interaction::None => {
                *background_color = BackgroundColor(Color::rgb(0.15, 0.15, 0.15));
            }
        }
    }
}

// Удаление всех элементов меню при запуске игры
fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<OnMainMenuScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}