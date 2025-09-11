use iced::{
    widget::{button, column, container, row, slider, text, pick_list, Space},
    Application, Command, Element, Length, Settings, Theme,
};
use std::time::{Duration, Instant};

fn main() -> iced::Result {
    CameraDemo::run(Settings::default())
}

#[derive(Debug, Clone)]
enum Message {
    FocusChanged(f32),
    IsoChanged(u32),
    ExposureChanged(f32),
    WhiteBalanceChanged(WhiteBalance),
    CapturePhoto,
    Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WhiteBalance {
    Auto,
    Daylight,
    Cloudy,
    Tungsten,
    Fluorescent,
}

impl std::fmt::Display for WhiteBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WhiteBalance::Auto => write!(f, "Auto"),
            WhiteBalance::Daylight => write!(f, "Daylight"),
            WhiteBalance::Cloudy => write!(f, "Cloudy"),
            WhiteBalance::Tungsten => write!(f, "Tungsten"),
            WhiteBalance::Fluorescent => write!(f, "Fluorescent"),
        }
    }
}

struct CameraDemo {
    focus: f32,
    iso: u32,
    exposure: f32,
    white_balance: WhiteBalance,
    fps: f32,
    last_frame: Instant,
    photos_captured: u32,
    camera_connected: bool,
    preview_width: u32,
    preview_height: u32,
}

impl Application for CameraDemo {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                focus: 50.0,
                iso: 400,
                exposure: 1.0 / 60.0,
                white_balance: WhiteBalance::Auto,
                fps: 0.0,
                last_frame: Instant::now(),
                photos_captured: 0,
                camera_connected: true, // Mock connection
                preview_width: 1280,
                preview_height: 720,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "CrabCamera Iced Demo - Professional Camera Controls".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FocusChanged(value) => {
                self.focus = value;
                // TODO: Send to CrabCamera
                println!("Focus changed to: {:.1}%", value);
            }
            Message::IsoChanged(value) => {
                self.iso = value;
                // TODO: Send to CrabCamera
                println!("ISO changed to: {}", value);
            }
            Message::ExposureChanged(value) => {
                self.exposure = value;
                // TODO: Send to CrabCamera
                println!("Exposure changed to: 1/{:.0}s", 1.0 / value);
            }
            Message::WhiteBalanceChanged(wb) => {
                self.white_balance = wb;
                // TODO: Send to CrabCamera
                println!("White balance changed to: {:?}", wb);
            }
            Message::CapturePhoto => {
                self.photos_captured += 1;
                // TODO: Trigger CrabCamera capture
                println!("Photo captured! Total: {}", self.photos_captured);
            }
            Message::Tick => {
                // Update FPS calculation
                let now = Instant::now();
                let elapsed = now.duration_since(self.last_frame);
                self.fps = 1.0 / elapsed.as_secs_f32();
                self.last_frame = now;
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let status_text = if self.camera_connected {
            format!("Camera Connected | {}x{} | {:.1} FPS", 
                   self.preview_width, self.preview_height, self.fps)
        } else {
            "Camera Disconnected".to_string()
        };

        let camera_preview = container(
            text("📷 Live Camera Preview")
                .size(24)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
        )
        .width(400)
        .height(300)
        .center_x()
        .center_y();

        let focus_control = column![
            text("Focus Control").size(16),
            slider(0.0..=100.0, self.focus, Message::FocusChanged)
                .step(1.0),
            text(format!("{:.0}%", self.focus))
                .size(14)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
        ].spacing(8);

        let iso_control = column![
            text("ISO Sensitivity").size(16),
            slider(100..=3200, self.iso, |value| Message::IsoChanged(value))
                .step(100u32),
            text(format!("ISO {}", self.iso))
                .size(14)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
        ].spacing(8);

        let exposure_control = column![
            text("Shutter Speed").size(16),
            slider(1.0/2000.0..=1.0/15.0, self.exposure, Message::ExposureChanged)
                .step(0.001),
            text(format!("1/{:.0}s", 1.0 / self.exposure))
                .size(14)
                .horizontal_alignment(iced::alignment::Horizontal::Center),
        ].spacing(8);

        let wb_options = vec![
            WhiteBalance::Auto,
            WhiteBalance::Daylight,
            WhiteBalance::Cloudy,
            WhiteBalance::Tungsten,
            WhiteBalance::Fluorescent,
        ];

        let white_balance_control = column![
            text("White Balance").size(16),
            pick_list(wb_options, Some(self.white_balance), Message::WhiteBalanceChanged)
                .width(Length::Fill),
        ].spacing(8);

        let capture_button = button(
            text("📸 Capture Photo")
                .size(18)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
        )
        .width(200)
        .height(50)
        .on_press(Message::CapturePhoto);

        let stats_panel = column![
            text("Performance Metrics").size(16).style(iced::Color::from_rgb8(100, 149, 237)),
            text(format!("Photos Captured: {}", self.photos_captured)).size(14),
            text(format!("Memory Usage: 28.4 MB")).size(14),
            text(format!("CPU Usage: 12.3%")).size(14),
            text(format!("Frame Latency: 16.7ms")).size(14),
        ].spacing(4);

        let controls_panel = column![
            text("Camera Controls")
                .size(20)
                .style(iced::Color::from_rgb8(100, 149, 237)),
            focus_control,
            Space::with_height(16),
            iso_control,
            Space::with_height(16),
            exposure_control,
            Space::with_height(16),
            white_balance_control,
            Space::with_height(24),
            capture_button,
            Space::with_height(24),
            stats_panel,
        ].spacing(8).width(250);

        let main_content = row![
            camera_preview,
            Space::with_width(32),
            controls_panel,
        ].spacing(16);

        let app_layout = column![
            text("CrabCamera Professional Controls")
                .size(24)
                .style(iced::Color::from_rgb8(70, 130, 180))
                .horizontal_alignment(iced::alignment::Horizontal::Center),
            text(status_text)
                .size(14)
                .style(iced::Color::from_rgb8(46, 204, 113))
                .horizontal_alignment(iced::alignment::Horizontal::Center),
            Space::with_height(16),
            main_content,
        ].spacing(8);

        container(app_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(24)
            .into()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        // Update FPS counter every frame
        iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick)
    }
}