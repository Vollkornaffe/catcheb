use std::collections::BTreeMap;

use nalgebra as na;

use quicksilver::{
    geom::{Circle, Rectangle, Transform, Vector},
    graphics::{
        //blend::{BlendEquation, BlendFunction, BlendMode, BlendOperation, BlendFactor, BlendChannel, BlendInput},
        Color,
        Graphics,
    },
};

use comn::{
    game::run::{BULLET_RADIUS, FOOD_MAX_LIFETIME, FOOD_SIZE, TURRET_RADIUS},
    geom,
    util::join,
};

use crate::view::Resources;

pub fn interp_entities<'a>(
    state: &'a comn::Game,
    next_entities: &'a BTreeMap<comn::EntityId, (comn::GameTime, comn::Entity)>,
    time: comn::GameTime,
) -> impl Iterator<Item = comn::Entity> + 'a {
    join::full_join(state.entities.iter(), next_entities.iter()).filter_map(
        move |item| match item {
            join::Item::Left(_, entity) => Some(entity.clone()),
            join::Item::Right(_, _) => None,
            join::Item::Both(_, entity, (next_time, next_entity)) => {
                let tau = (time - state.game_time()) / (next_time - state.game_time());
                Some(entity.interp(next_entity, tau))
            }
        },
    )
}

pub fn interp_entity(
    state: &comn::Game,
    next_entities: &BTreeMap<comn::EntityId, (comn::GameTime, comn::Entity)>,
    time: comn::GameTime,
    entity_id: comn::EntityId,
) -> Option<comn::Entity> {
    match (
        state.entities.get(&entity_id),
        next_entities.get(&entity_id),
    ) {
        (Some(entity), None) => Some(entity.clone()),
        (Some(entity), Some((next_time, next_entity))) => {
            let tau = (time - state.game_time()) / (next_time - state.game_time());
            Some(entity.interp(next_entity, tau))
        }
        (None, _) => None,
    }
}

pub fn render_game(
    gfx: &mut Graphics,
    resources: &mut Resources,
    state: &comn::Game,
    next_entities: &BTreeMap<comn::EntityId, (comn::GameTime, comn::Entity)>,
    time: comn::GameTime,
    my_player_id: comn::PlayerId,
    camera_transform: Transform,
) -> quicksilver::Result<()> {
    {
        gfx.set_transform(camera_transform);
        let map_size: mint::Vector2<f32> = state.settings.size.into();
        let map_rect = Rectangle::new(Vector::new(0.0, 0.0), map_size.into());
        //gfx.fill_rect(&map_rect, Color::from_rgba(204, 255, 204, 1.0));
        gfx.fill_rect(&map_rect, Color::WHITE);
        gfx.stroke_rect(&map_rect, Color::BLACK);
    }

    {
        /*gfx.set_blend_mode(Some(BlendMode {
            equation: BlendEquation::Same(BlendOperation::Add),
            function: BlendFunction::Same {
                source: BlendFactor::Color {
                    input: BlendInput::Source,
                    channel: BlendChannel::Alpha,
                    is_inverse: false,
                },
                destination: BlendFactor::Color {
                    input: BlendInput::Source,
                    channel: BlendChannel::Alpha,
                    is_inverse: true,
                },
            },
            ..BlendMode::default()
        }));*/

        /*for (_, entity) in state.entities.iter() {
            match entity {
                comn::Entity::Turret(turret) => {
                    let origin: mint::Vector2<f32> = turret.pos.coords.into();

                    if let Some(target) = turret.target.and_then(|id| state.entities.get(&id)) {
                        let target_pos: mint::Vector2<f32> = target.pos(time).coords.into();
                        gfx.stroke_path(&[origin.into(), target_pos.into()], Color::from_rgba(255, 0, 0, 1.0));
                    }
                }
                _ => (),
            }
        }*/

        //gfx.set_blend_mode(Some(Default::default()));
    }

    for entity in interp_entities(state, next_entities, time) {
        match entity {
            comn::Entity::Player(player) => {
                render_player(
                    gfx,
                    resources,
                    state,
                    next_entities,
                    time,
                    my_player_id,
                    camera_transform,
                    &player.to_view(),
                )?;
            }
            comn::Entity::PlayerView(player) => {
                render_player(
                    gfx,
                    resources,
                    state,
                    next_entities,
                    time,
                    my_player_id,
                    camera_transform,
                    &player,
                )?;
            }
            comn::Entity::DangerGuy(danger_guy) => {
                let origin: mint::Vector2<f32> =
                    (danger_guy.pos(time) - danger_guy.size / 2.0).coords.into();
                let size: mint::Vector2<f32> = danger_guy.size.into();
                let rect = Rectangle::new(origin.into(), size.into());
                gfx.set_transform(camera_transform);

                // Awesome Hirsch, add back in once we have more images!
                /*let frame = pareen::constant(0)
                    .switch(danger_guy.wait_time - 0.6, 1)
                    .switch(danger_guy.wait_time - 0.4, 2)
                    .switch(danger_guy.wait_time - 0.2, 3)
                    .seq(
                        danger_guy.wait_time,
                        pareen::fun(|tau| 3 + (tau * danger_guy.speed / 40.0) as usize % 4),
                    )
                    .repeat(danger_guy.period() / 2.0)
                    .eval(danger_guy.tau(time)) as f32;

                let flip = danger_guy
                    .dir(time)
                    .normalize()
                    .dot(&comn::Vector::new(1.0, 0.0))
                    > 0.7;
                let sub_rect = if flip {
                    Rectangle::new(
                        Vector::new(17.0, 16.0 * frame + 1.0),
                        Vector::new(-16.0, 16.0),
                    )
                } else {
                    Rectangle::new(
                        Vector::new(1.0, 16.0 * frame + 1.0),
                        Vector::new(16.0, 16.0),
                    )
                };

                gfx.draw_subimage(&resources.hirsch, sub_rect, rect);*/

                let color = if danger_guy.is_hot {
                    Color::RED
                } else {
                    Color::CYAN
                };

                gfx.fill_rect(&rect, color);
                gfx.stroke_rect(&rect, Color::BLACK);
            }
            comn::Entity::Bullet(bullet) => {
                let origin: mint::Vector2<f32> = bullet.pos(time).coords.into();
                let circle = Circle::new(origin.into(), BULLET_RADIUS);
                let color = if bullet.owner == Some(my_player_id) {
                    Color::ORANGE
                } else {
                    Color::MAGENTA
                };
                gfx.set_transform(camera_transform);
                gfx.fill_circle(&circle, color);
                gfx.stroke_circle(&circle, Color::BLACK);
            }
            comn::Entity::Turret(turret) => {
                let origin: mint::Vector2<f32> = turret.pos.coords.into();
                let color = if turret.target.is_some() {
                    Color::RED
                } else {
                    Color::from_rgba(150, 150, 150, 1.0)
                };
                let circle = Circle::new(origin.into(), TURRET_RADIUS);
                gfx.set_transform(camera_transform);
                gfx.fill_circle(&circle, color);
                gfx.stroke_circle(&circle, Color::BLACK);

                let angle = turret.angle;

                gfx.set_transform(
                    Transform::rotate(angle.to_degrees())
                        .then(Transform::translate(origin.into()))
                        .then(camera_transform),
                );

                let rect = Rectangle::new(Vector::new(0.0, -5.0), Vector::new(40.0, 10.0));

                gfx.fill_rect(&rect, Color::BLACK);
            }
            comn::Entity::Wall(wall) => {
                let transform = rect_to_transform(&wall.rect.to_rect());
                let rect = Rectangle::new(Vector::new(-0.5, -0.5), Vector::new(1.0, 1.0));
                gfx.set_transform(transform.then(camera_transform));
                gfx.fill_rect(&rect, Color::from_rgba(170, 170, 170, 1.0));
                gfx.stroke_rect(&rect, Color::BLACK);
            }
            comn::Entity::FoodSpawn(spawn) => {
                let origin: mint::Vector2<f32> = spawn.pos.coords.into();
                let transform = rect_to_transform(&spawn.rect(time));

                if spawn.has_food {
                    let rect = Rectangle::new(Vector::new(-0.5, -0.5), Vector::new(1.0, 1.0));
                    gfx.set_transform(transform.then(camera_transform));
                    gfx.fill_rect(&rect, Color::ORANGE);
                    gfx.stroke_rect(&rect, Color::BLACK);
                }

                let circle = Circle::new(origin.into(), FOOD_SIZE * 1.3);
                gfx.set_transform(camera_transform);
                gfx.stroke_circle(&circle, Color::BLACK);
            }
            comn::Entity::Food(food) => {
                let transform = rect_to_transform(&food.rect(time));

                let rect = Rectangle::new(Vector::new(-0.5, -0.5), Vector::new(1.0, 1.0));
                gfx.set_transform(transform.then(camera_transform));

                // TODO: Blending's-a not working -- user error or not?
                let alpha = pareen::constant(1.0)
                    .seq_ease_out(0.9, pareen::easer::functions::Sine, 0.1, 0.0)
                    .squeeze(food.start_time..=food.start_time + FOOD_MAX_LIFETIME)
                    .eval(time);
                gfx.fill_rect(
                    &rect,
                    Color {
                        r: 1.0,
                        g: 1.0 - 0.5 * alpha,
                        b: 1.0 - alpha,
                        a: 1.0,
                    },
                );
                gfx.stroke_rect(
                    &rect,
                    Color {
                        r: 1.0 - alpha,
                        g: 1.0 - alpha,
                        b: 1.0 - alpha,
                        a: 1.0,
                    },
                );
            }
        }
    }

    gfx.set_transform(Transform::IDENTITY);

    Ok(())
}

fn render_player(
    gfx: &mut Graphics,
    resources: &mut Resources,
    state: &comn::Game,
    next_entities: &BTreeMap<comn::EntityId, (comn::GameTime, comn::Entity)>,
    time: comn::GameTime,
    my_player_id: comn::PlayerId,
    camera_transform: Transform,
    player: &comn::PlayerView,
) -> quicksilver::Result<()> {
    let pos: mint::Vector2<f32> = player.pos.coords.into();

    let color = if player.owner == my_player_id {
        Color::BLUE
    } else {
        Color::from_rgba(148, 0, 211, 1.0)
    };

    let transform = rect_to_transform(&player.rect());
    let rect = Rectangle::new(Vector::new(-0.5, -0.5), Vector::new(1.0, 1.0));

    gfx.set_transform(transform.then(camera_transform));
    gfx.fill_rect(&rect, color);
    gfx.stroke_rect(&rect, Color::BLACK);

    let nose = Rectangle::new(Vector::new(0.5, -0.1), Vector::new(0.2, 0.2));
    gfx.set_transform(transform.then(camera_transform));
    gfx.fill_rect(&nose, Color::CYAN);

    gfx.set_transform(camera_transform);

    if let Some(hook) = player.hook.as_ref() {
        render_hook(gfx, state, next_entities, time, player.pos, hook)?;
    }

    resources
        .font
        .draw(gfx, &player.owner.0.to_string(), Color::WHITE, pos.into())?;

    Ok(())
}

fn render_hook(
    gfx: &mut Graphics,
    state: &comn::Game,
    next_entities: &BTreeMap<comn::EntityId, (comn::GameTime, comn::Entity)>,
    time: comn::GameTime,
    pos: comn::Point,
    hook: &comn::Hook,
) -> quicksilver::Result<()> {
    let (hook_pos, dead) = match hook {
        comn::Hook::Shooting {
            pos: hook_pos,
            vel: _,
            time_left: _,
        } => (*hook_pos, false),
        comn::Hook::Attached { target, offset } => {
            let hook_pos = interp_entity(state, next_entities, time, *target)
                .map_or(pos, |interp_target| interp_target.pos(time) + offset);
            (hook_pos, false)
        }
        comn::Hook::Contracting { pos: hook_pos } => (*hook_pos, true),
    };

    let a: mint::Vector2<f32> = pos.coords.into();
    let b: mint::Vector2<f32> = hook_pos.coords.into();
    if !dead {
        gfx.stroke_path(&[a.into(), b.into()], Color::from_rgba(100, 100, 100, 1.0));
        gfx.fill_circle(
            &Circle::new(b.into(), 12.0),
            Color::from_rgba(50, 200, 50, 1.0),
        );
    } else {
        gfx.stroke_path(&[a.into(), b.into()], Color::from_rgba(100, 100, 100, 1.0));
        gfx.stroke_circle(
            &Circle::new(b.into(), 7.0),
            Color::from_rgba(100, 100, 100, 1.0),
        );
    }

    Ok(())
}

fn rect_to_transform(rect: &geom::Rect) -> Transform {
    let m_column: mint::ColumnMatrix3<f32> = na::Matrix3::new(
        rect.x_edge.x,
        rect.y_edge.x,
        rect.center.x,
        rect.x_edge.y,
        rect.y_edge.y,
        rect.center.y,
        0.0,
        0.0,
        1.0,
    )
    .into();
    let m_row: mint::RowMatrix3<f32> = m_column.into();

    m_row.into()
}