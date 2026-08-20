use std::{
    io,
    net::{SocketAddr, TcpStream},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const DSH_PORT: u16 = 3080;

pub const DSH_URL: &str = "http://127.0.0.1:3080";

/// 检查当前桌面应用环境是否可以执行 `dsh`。
///
/// 这里只验证命令能够被解析并启动，不要求 `--version` 返回成功，
/// 这样可以兼容没有实现标准版本参数的 DSH 版本。
pub fn is_available() -> bool {
    Command::new("dsh")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

const CONNECT_TIMEOUT: Duration = Duration::from_millis(300);

const START_TIMEOUT: Duration = Duration::from_secs(10);

const POLL_INTERVAL: Duration = Duration::from_millis(200);

/// 检查 127.0.0.1:3080 是否有程序监听.
///
/// 注意：
///
/// 这里只能证明端口已经被占用，
/// 不能证明监听者一定是 DSH。
pub fn port_is_open() -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], DSH_PORT));

    TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT).is_ok()
}

/// 启动 DSH Web。
///
/// 实际执行：
///
/// dsh web --no-open
pub fn start() -> io::Result<Child> {
    println!("Starting DSH Web...");

    let mut child = Command::new("dsh").args(["web", "--no-open"]).spawn()?;

    let deadline = Instant::now() + START_TIMEOUT;

    loop {
        //
        // Web Server 已经可以连接。
        //
        if port_is_open() {
            println!("DSH Web started at {DSH_URL}");

            return Ok(child);
        }

        //
        // 如果 DSH 自己提前退出，
        // 不要继续等完整的 10 秒超时。
        //
        if let Some(status) = child.try_wait()? {
            return Err(io::Error::other(format!(
                "DSH Web exited before becoming ready: {status}"
            )));
        }

        //
        // 启动超时。
        //
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();

            return Err(io::Error::other(
                "DSH Web failed to start within 10 seconds",
            ));
        }

        thread::sleep(POLL_INTERVAL);
    }
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
