use bracket_lib::prelude::*;

enum GameMode {
    Menu,
    Playing,
    End,
}

const SCREEN_WIDTH: i32 = 50; // 游戏宽度
const SCREEN_HEIGHT: i32 = 30; // 游戏高度
const FRAME_DURATION: f32 = 100.0; // 刷新速率
const GRAVITY: f32 = 0.15; // 重力
const FLAP_STRENGTH: f32 = -1.5; // 跳跃力度

struct Player {
    x: f32, // 使用 f32 以获得更平滑的运动
    y: f32,
    velocity: f32,
}

impl Player {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            velocity: 0.0,
        }
    }

    fn render(&self, ctx: &mut BTerm) {
        ctx.set(5, self.y as i32, YELLOW, BLACK, to_cp437('■'));
    }

    fn gravity_and_move(&mut self) {
        self.velocity += GRAVITY;
        self.y += self.velocity;

        if self.y < 0.0 {
            self.y = 0.0;
            self.velocity = 0.0;
        }

        // 确保不会掉出屏幕底部
        if self.y > SCREEN_HEIGHT as f32 {
            self.y = SCREEN_HEIGHT as f32;
        }
    }

    fn flap(&mut self) {
        self.velocity = FLAP_STRENGTH;
    }
}

struct State {
    player: Player,
    frame_time: f32,
    obstacle: Obstacle,
    mode: GameMode,
    score: i32,
}

impl State {
    fn new() -> Self {
        Self {
            player: Player::new(5.0, 20.0),
            frame_time: 0.0,
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
            mode: GameMode::Menu,
            score: 0,
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;

        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move();
            self.obstacle.move_obstacle();
        }

        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }

        self.player.render(ctx);
        ctx.print(0, 0, "Press SPACE to flap.");
        ctx.print(0, 1, &format!("Score: {}", self.score));

        self.obstacle.render(ctx, self.player.x as i32);

        // 增加分数逻辑
        if self.player.x as i32 > self.obstacle.x {
            self.score += 1;
            self.obstacle = Obstacle::new(SCREEN_WIDTH, self.score);
        }

        // 更新死亡逻辑
        if self.obstacle.hit_obstacle(&self.player) || self.player.y >= SCREEN_HEIGHT as f32 {
            self.mode = GameMode::End;
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(5.0, 20.0);
        self.frame_time = 0.0;
        self.obstacle = Obstacle::new(SCREEN_WIDTH, 0);
        self.mode = GameMode::Playing;
        self.score = 0;
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to Flappy Dragon");
        ctx.print_color(0, 8, YELLOW, BLACK, "(P) Play Game");
        ctx.print_color(0, 9, YELLOW, BLACK, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You are dead");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_color(0, 8, RED, BLACK, "(P) Play again");
        ctx.print_color(0, 9, RED, BLACK, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::End => self.dead(ctx),
        }
    }
}

struct Obstacle {
    x: i32,
    gap_y: i32,
    size: i32,
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Obstacle {
            x,
            gap_y: random.range(10, 30), // 随机生成间隙位置
            size: i32::max(2, 20 - score),
        }
    }

    fn move_obstacle(&mut self) {
        self.x -= 1; // 每帧向左移动
        // 当障碍物超出屏幕时，重新生成
        if self.x < 0 {
            self.x = SCREEN_WIDTH;
        }
    }

    fn render(&self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x;
        let half_size = self.size / 2;

        // 确保上部障碍物渲染
        for y in 0..self.gap_y - half_size {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('▬'));
        }

        // 确保下部障碍物渲染
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(screen_x, y, RED, BLACK, to_cp437('▬'));
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.x as i32 >= self.x && player.x as i32 <= self.x + 1; // 判断 X 坐标是否重合
        let player_above_gap = player.y < self.gap_y as f32 - half_size as f32;
        let player_below_gap = player.y > self.gap_y as f32 + half_size as f32;

        // 如果 X 坐标重合且 Y 坐标在上下障碍物之间，则判定为撞上
        does_x_match && (player_above_gap || player_below_gap)
    }
}

pub fn startup() -> BError {
    // let context = BTermBuilder::simple80x50();
    let context = BTermBuilder::simple(50, 30).unwrap()
        .with_title("Flappy Dragon")
        .build()?;

    main_loop(context, State::new())
}
