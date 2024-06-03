#![no_std]
#![no_main]

use rustngo_lib::*;

const FRAME_MS: u32 = 15;
const MAX_SCORE: u8 = 10;
const TONE: u32 = 440;

const SCREEN_WIDTH: u8 = 128;
const SCREEN_HEIGHT: u8 = 64;
const PADDLE_WIDTH: u8 = 2;
const PADDLE_HEIGHT: u8 = 10;
const PADDLE_BOUND: i8 = 2;

const P1_START_X: i8 = 5;
const P1_START_Y: i8 = 20;
const P1_SPEED: i8 = 2;
const P2_START_X: i8 = 121;
const P2_START_Y: i8 = 20;
const P2_SPEED: i8 = 1;

const BALL_START_X: i8 = P1_START_X + PADDLE_WIDTH as i8 + 1;
const BALL_START_Y: i8 = P1_START_Y + 4;
const BALL_SIZE: u8 = 2;
const BALL_SPEED: i8 = 2;

struct Paddle {
    x: i8,
    y: i8,
    w: u8,
    h: u8,
    score: u8,
}

struct Ball {
    x: i8,
    y: i8,
    s: u8,
    xdir: i8,
    ydir: i8,
    speed: i8,
}

enum State {
    Title,
    Launch,
    Play,
    Win,
    Lose,
}

fn reset(p1: &mut Paddle, p2: &mut Paddle, ball: &mut Ball) {
    p1.x = P1_START_X;
    p1.y = P1_START_Y;
    p2.x = P2_START_X;
    p2.y = P2_START_Y;
    ball.x = BALL_START_X;
    ball.y = BALL_START_Y;
    ball.xdir = 1;
    ball.ydir = 1;
}

fn play_hit_tone() {
    play_tone(TONE);
    delay(10);
    play_tone(0);
}

fn play_gameover_tone() {
    for _ in 0..3 {
        play_tone(TONE);
        delay(500);
        play_tone(0);
        delay(500);
    }
}

fn print_play_display(p1: &mut Paddle, p2: &mut Paddle, ball: &mut Ball) {
    display_clear();
    print_score(p1.score, (SCREEN_WIDTH - 1) as i8 / 2 - 13, 10);
    print_score(p2.score, (SCREEN_WIDTH - 1) as i8 / 2 + 10, 10);
    draw_net();
    display_draw_rect(p1.x as u8, p1.y as u8, p1.w, p1.h, true);
    display_draw_rect(p2.x as u8, p2.y as u8, p2.w, p2.h, true);
    display_draw_rect(ball.x as u8, ball.y as u8, ball.s, ball.s, true);
    display_update();
}

fn print_score(score: u8, x: i8, y: i8) {
    // Yeah I know (string formatting is annoying in no-std)
    let s = match score {
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        9 => "9",
        _ => "0",
    };
    display_print(x as u8, y as u8, s);
}

fn draw_net() {
    let w = 2;
    let h = 5;
    let x = SCREEN_WIDTH / 2 - w / 2;
    let y = 5;

    for i in 0..6 {
        display_draw_rect(x, y + i * 10, w, h, true);
    }
}

fn handle_ball_move(ball: &mut Ball) {
    ball.x += ball.xdir * ball.speed;
    ball.y += ball.ydir * ball.speed;
}

fn handle_p2_move(p2: &mut Paddle, ball: &Ball) {
    if ball.y >= p2.y + PADDLE_HEIGHT as i8 / 2 {
        p2.y += P2_SPEED;
    } else {
        p2.y -= P2_SPEED;
    }
}

fn handle_paddle_collision(p1: &Paddle, p2: &Paddle, ball: &mut Ball) {
    if ball.x >= p2.x - BALL_SIZE as i8
        && ball.x <= p2.x + PADDLE_WIDTH as i8 + BALL_SIZE as i8
        && ball.y >= p2.y - BALL_SIZE as i8
        && ball.y <= p2.y + PADDLE_HEIGHT as i8
    {
        ball.xdir = -1;
        play_hit_tone();
    }

    if ball.x <= p1.x + BALL_SIZE as i8
        && ball.x >= p1.x - PADDLE_WIDTH as i8 - BALL_SIZE as i8
        && ball.y >= p1.y - BALL_SIZE as i8
        && ball.y <= p1.y + PADDLE_HEIGHT as i8
    {
        ball.xdir = 1;
        play_hit_tone();
    }
}

fn handle_y_collision(ball: &mut Ball) {
    if ball.y >= SCREEN_HEIGHT as i8 || ball.y <= 0 {
        ball.ydir *= -1;
    }
}

fn handle_score(p1: &mut Paddle, p2: &mut Paddle, ball: &mut Ball, state: &mut State) {
    if ball.x >= (SCREEN_WIDTH - BALL_SIZE) as i8 || ball.x <= 0 {
        if ball.x <= 0 {
            p2.score += 1;
        } else {
            p1.score += 1;
        }

        play_tone(TONE);
        delay(1000);
        play_tone(0);

        reset(p1, p2, ball);
        *state = State::Launch;
    }
}

fn handle_title(state: &mut State) {
    display_clear();
    display_print(SCREEN_WIDTH / 2 - 10, 30, "PONG");
    display_print(SCREEN_WIDTH / 2 - 20, 40, "PRESS B");
    display_update();

    while !is_pressed('B') {}
    *state = State::Launch;
}

fn handle_launch(p1: &mut Paddle, p2: &mut Paddle, ball: &mut Ball, state: &mut State) {
    if is_pressed('U') && p1.y > PADDLE_BOUND {
        p1.y -= P1_SPEED;
        ball.y -= P1_SPEED;
    } else if is_pressed('D') && p1.y < SCREEN_HEIGHT as i8 - PADDLE_BOUND - PADDLE_HEIGHT as i8 {
        p1.y += P1_SPEED;
        ball.y += P1_SPEED;
    } else if is_pressed('A') {
        *state = State::Play;
    }

    print_play_display(p1, p2, ball);
}

fn handle_play(p1: &mut Paddle, p2: &mut Paddle, ball: &mut Ball, state: &mut State) {
    handle_ball_move(ball);
    handle_p2_move(p2, ball);
    handle_y_collision(ball);
    handle_paddle_collision(p1, p2, ball);
    handle_score(p1, p2, ball, state);

    if p1.score >= MAX_SCORE {
        p1.score = 0;
        p2.score = 0;
        *state = State::Win;
        return;
    } else if p2.score >= MAX_SCORE {
        p1.score = 0;
        p2.score = 0;
        *state = State::Lose;
        return;
    }

    if is_pressed('U') && p1.y > PADDLE_BOUND {
        p1.y -= P1_SPEED;
    } else if is_pressed('D') && p1.y < SCREEN_HEIGHT as i8 - PADDLE_BOUND - PADDLE_HEIGHT as i8 {
        p1.y += P1_SPEED;
    }

    print_play_display(p1, p2, ball);
}

fn handle_win(state: &mut State) {
    display_clear();
    display_print(SCREEN_WIDTH / 2 - 23, 35, "YOU WIN!");
    display_update();

    play_gameover_tone();
    *state = State::Title;
}

fn handle_lose(state: &mut State) {
    display_clear();
    display_print(SCREEN_WIDTH / 2 - 30, 35, "YOU LOSE!");
    display_update();

    play_gameover_tone();
    *state = State::Title;
}

#[no_mangle]
fn game() {
    console_print("Starting Pong...");

    let mut p1 = Paddle {
        x: P1_START_X,
        y: P1_START_Y,
        w: PADDLE_WIDTH,
        h: PADDLE_HEIGHT,
        score: 0,
    };
    let mut p2 = Paddle {
        x: P2_START_X,
        y: P2_START_Y,
        w: PADDLE_WIDTH,
        h: PADDLE_HEIGHT,
        score: 0,
    };
    let mut ball = Ball {
        x: BALL_START_X,
        y: BALL_START_Y,
        s: BALL_SIZE,
        xdir: 1,
        ydir: 1,
        speed: BALL_SPEED,
    };

    let mut state = State::Title;
    loop {
        match state {
            State::Title => handle_title(&mut state),
            State::Launch => handle_launch(&mut p1, &mut p2, &mut ball, &mut state),
            State::Play => handle_play(&mut p1, &mut p2, &mut ball, &mut state),
            State::Win => handle_win(&mut state),
            State::Lose => handle_lose(&mut state),
        }

        delay(FRAME_MS);
    }
}
