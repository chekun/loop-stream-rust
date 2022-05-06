mod config;
mod video;
use std::process::{Command, Stdio};
use std::fs;

fn main() {
    let config = config::parse_config();

    let mut videos: Vec<video::Video> = Vec::new();
    let mut playlist: Vec<String> = Vec::new();
    let mut total_duration: f64 = 0.0;
    // 1. Get video metadata
    for v in config.input.episodes {
        let output = Command::new(&config.ffprob).args([
            "-v",
			"error",
			"-show_entries",
			"format=duration",
			"-of",
			"default=noprint_wrappers=1:nokey=1",
			v.as_str()
        ]).output().expect("failed to execute ffprobe");
        playlist.push(std::format!("file {}", v));
        let output = String::from_utf8(output.stdout).unwrap();
        let output = output.replace("\n", "");
        let output = match output.parse::<f64>() {
            Ok(output) => output,
            Err(error) => panic!("failed to convert ffprobe output {} to f64, {}", output, error)
        };
        total_duration += output;
        let filename: Vec<&str> = v.split("/").collect();
        let filename = filename[filename.len() - 1];
        let filename = filename.replace(".flv", "").replace(".mp4", "");
        videos.push(video::Video{
            name: String::from(filename),
            duration: output,
        });
    }

    // 2. Compose playlist and drawtext params
    let mut drawtexts: Vec<String> = Vec::new();
    let mut offset = 0.0;
    let loop_one_time = std::format!("{}", total_duration);
    for v in videos {
        let from = std::format!("{}", offset);
        let to = std::format!("{}", offset + v.duration);
        let draw_param = std::format!(
            "drawtext=fontfile='{}': text='{}{}':x={}:y={}:fontcolor={}:fontsize={}:box=1:boxcolor=0x00000099:enable='between(mod(t,{}),{},{})'",
            config.input.title.font,
            config.input.title.prefix,
            v.name,
            config.input.title.x,
            config.input.title.y,
            config.input.title.color,
            config.input.title.size,
            loop_one_time,
            from, 
            to,
        );
        drawtexts.push(draw_param);
        offset += v.duration;
    }
    // 3. Write playlist file
    fs::write("playlist.txt", playlist.join("\n"))
        .expect("failed to write playlist file");

    // 4. Start streaming
    let feed = Command::new(&config.ffmpeg).args([
        "-hide_banner",
        "-loglevel",
		"quiet",
        "-nostats",
		"-stream_loop",
		"-1",
		"-f",
		"concat",
		"-re",
		"-safe",
		"0",
		"-i",
		"playlist.txt",
		"-f",
		"flv",
		"-",
    ]).stdout(Stdio::piped()).spawn();
    
   let stdout = match feed {
        Ok(child) => match child.stdout {
            Some(stdout) => stdout,
            None => panic!("No stdout for a command"),
        },
        Err(e) => panic!("failed to get stdout {}", e),
    };
    let mut push = Command::new(&config.ffmpeg).args([
        "-hide_banner",
		"-loglevel",
		"quiet",
        "-nostats",
		"-re",
		"-i",
		config.stage.image.as_str(),
		"-i",
		"-",
		"-filter_complex",
		std::format!(
			"[0:v] pad={}:{}[bg];[1:v]scale={}:{}[temp1];[bg][temp1] overlay={}:{}[temp2];[temp2]{}",
			config.stage.width.as_str(),
			config.stage.height.as_str(),
			config.input.rectangle[2].as_str(),
			config.input.rectangle[3].as_str(),
			config.input.rectangle[0].as_str(),
			config.input.rectangle[1].as_str(),
			drawtexts.join(",").as_str(),
		).as_str(),
		"-f",
		"flv",
		"-c:a",
		"copy",
		"-c:v",
		"libx264",
		config.output.stream_url.as_str(),
    ]).stdin(stdout).spawn().expect("failed to exec push command");
    println!("Streaming ......");
    push.wait().unwrap();
}
