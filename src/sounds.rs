use ggez::audio;
use ggez::audio::SoundSource;
use ggez::Context;

pub struct SoundSet {
	pub coin_collect: audio::Source,
}

impl SoundSet {
	pub fn new(ctx: &mut Context) -> SoundSet {
		let s = audio::Source::new(ctx, "/collect_coin.wav").unwrap();
		SoundSet {
			coin_collect: s,
		}
	}
}