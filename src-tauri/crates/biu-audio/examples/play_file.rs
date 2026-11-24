use std::time::Duration;

use biu_audio::Player;

fn main() {
    let path = std::env::args().nth(1).expect("usage: play_file <audio>");
    let player = Player::new().expect("audio device");
    player.load(&path);
    player.play();
    std::thread::sleep(Duration::from_millis(700));
    println!("after play: {:?}", player.snapshot());
    player.pause();
    std::thread::sleep(Duration::from_millis(200));
    println!("after pause: {:?}", player.snapshot());
    player.seek_ms(1_000);
    player.play();
    std::thread::sleep(Duration::from_millis(500));
    println!("after seek: {:?}", player.snapshot());
    player.stop();
    std::thread::sleep(Duration::from_millis(100));
    println!("after stop: {:?}", player.snapshot());
}
