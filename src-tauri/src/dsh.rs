use std::{
    io,
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use tauri::Url;

pub struct Server {
    pub child: Child,
    pub url: Url,
}

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(windows)]
fn dsh_command() -> Command {
    use std::os::windows::process::CommandExt;

    let mut command = Command::new("dsh");
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(not(windows))]
fn dsh_command() -> Command {
    Command::new("dsh")
}

/// 检查当前桌面应用环境是否可以执行 `dsh`。
///
/// 这里只验证命令能够被解析并启动，不要求 `--version` 返回成功，
/// 这样可以兼容没有实现标准版本参数的 DSH 版本。
pub fn is_available() -> bool {
    dsh_command()
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

const START_TIMEOUT: Duration = Duration::from_secs(10);

const POLL_INTERVAL: Duration = Duration::from_millis(200);

pub fn start() -> io::Result<Server> {
    println!("Starting DSH Web...");

    let mut child = dsh_command()
        .args(["web", "--no-open", "--port", "0"])
        .stdout(Stdio::piped())
        .spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("DSH Web stdout is unavailable"))?;
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if sender.send(line).is_err() {
                break;
            }
        }
    });

    let deadline = Instant::now() + START_TIMEOUT;

    loop {
        if let Some(status) = child.try_wait()? {
            return Err(io::Error::other(format!(
                "DSH Web exited before becoming ready: {status}"
            )));
        }

        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();

            return Err(io::Error::other(
                "DSH Web failed to start within 10 seconds",
            ));
        }

        match receiver.recv_timeout(POLL_INTERVAL) {
            Ok(Ok(line)) => {
                if let Some(url) = parse_dsh_url(&line) {
                    println!("DSH Web started at {url}");
                    return Ok(Server { child, url });
                }
            }
            Ok(Err(error)) => return Err(error),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => thread::sleep(POLL_INTERVAL),
        }
    }
}

fn parse_dsh_url(line: &str) -> Option<Url> {
    let url = line.strip_prefix("dsh web: ")?.trim().parse::<Url>().ok()?;

    (url.scheme() == "http" && url.host_str() == Some("127.0.0.1") && url.port()? != 0)
        .then_some(url)
}

/// 关闭由 DSH Desktop 启动的 DSH。
pub fn stop(child: &mut Child) {
    println!("Stopping DSH Web...");

    if let Err(error) = child.kill() {
        eprintln!("Failed to stop DSH Web: {error}");
    }

    if let Err(error) = child.wait() {
        eprintln!("Failed to wait for DSH Web: {error}");
    }
}
