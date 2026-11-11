import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

WIDTH, HEIGHT = 1280, 900
WORKSPACE = Path(__file__).resolve().parents[1]


def tool(name):
    prefix = os.environ.get("ESTELA_TEST_PREFIX")
    candidate = Path(prefix) / "bin" / name if prefix else None
    return str(candidate) if candidate and candidate.exists() else shutil.which(name) or name


def run(wayland):
    artifacts = WORKSPACE / "target" / ("recorder-wayland" if wayland else "recorder-x11")
    artifacts.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="recorder-ui-", dir=WORKSPACE / "target") as temporary:
        env = os.environ.copy()
        env.pop("WAYLAND_DISPLAY", None)
        env.update(XDG_RUNTIME_DIR=temporary, XDG_CONFIG_HOME=temporary,
                   DBUS_SESSION_BUS_ADDRESS="unix:path=/nonexistent-estela-test-bus",
                   ICED_BACKEND=os.environ.get("ICED_BACKEND", "wgpu"), LC_ALL="es_DO.UTF-8")
        processes = []
        with open(artifacts / "session.log", "w") as log:
            try:
                read_fd, write_fd = os.pipe()
                server = subprocess.Popen([tool("Xvfb"), "-displayfd", str(write_fd),
                                           "-screen", "0", "1280x900x24", "-nolisten", "tcp"],
                                          pass_fds=(write_fd,), stdout=log, stderr=log)
                processes.append(server)
                os.close(write_fd)
                with os.fdopen(read_fd) as display:
                    env["DISPLAY"] = ":" + display.readline().strip()
                if wayland:
                    start_wayland(env, temporary, log, processes)
                executable = os.environ.get("ESTELA_TEST_APP", WORKSPACE / "target/debug/estela-app")
                app_env = env.copy()
                if wayland:
                    app_env["WAYLAND_DEBUG"] = "client"
                app = subprocess.Popen([str(executable)], env=app_env, stdout=log, stderr=log)
                processes.append(app)
                time.sleep(2)
                frame = capture(env, artifacts / "initial.png")
                initial = frame
                click(frame, env, -300, 0)
                time.sleep(0.8)
                frame = capture(env, artifacts / "sources.png")
                if wayland:
                    click(frame, env, -135, 76)
                    time.sleep(0.5)
                    frame = capture(env, artifacts / "windows.png")
                    click(frame, env, -200, 190)
                    time.sleep(1)
                    frame = capture(env, artifacts / "window-picker-failure.png")
                    assert frame == initial, "Canceled source selection moved the recorder"
                click(frame, env, -165, 0)
                time.sleep(0.8)
                frame = capture(env, artifacts / "microphone.png")
                click(frame, env, 22, 0)
                time.sleep(0.8)
                frame = capture(env, artifacts / "menu.png")
                click(frame, env, -200, 110)
                time.sleep(0.8)
                frame = capture(env, artifacts / "settings.png")
                click(frame, env, 22, 0)
                time.sleep(0.5)
                frame = capture(env, artifacts / "menu-again.png")
                click(frame, env, -200, 150)
                time.sleep(0.8)
                frame = capture(env, artifacts / "shortcuts.png")
                assert frame == initial, "Opening panels moved the recorder"
                click(frame, env, 22, 0)
                time.sleep(0.5)
                frame = capture(env, artifacts / "menu-exit.png")
                if wayland:
                    frame = check_wayland_drag_and_input(frame, env, artifacts, log, processes)
                click(frame, env, -200, 190)
                assert app.wait(timeout=5) == 0, "Quit did not exit cleanly"
                contents = (artifacts / "session.log").read_text()
                if wayland:
                    assert contents.count(".get_xdg_surface(") == 1, "Recorder surface was recreated"
                for error in ["not a valid new object id", "Io error", "protocol error", "panicked"]:
                    assert error not in contents, f"Native protocol failure: {error}"
                print(f"Recorder controls and all option panels passed: {artifacts}")
            finally:
                for process in reversed(processes):
                    if process.poll() is None:
                        process.terminate()
                        process.wait(timeout=10)


def check_wayland_drag_and_input(frame, env, artifacts, log, processes):
    x, y = frame
    subprocess.run([tool("xdotool"), "mousemove", str(x - 382), str(y), "mousedown", "1"], env=env, check=True)
    time.sleep(0.1)
    subprocess.run([tool("xdotool"), "mousemove", str(x - 262), str(y + 80)], env=env, check=True)
    time.sleep(0.2)
    subprocess.run([tool("xdotool"), "mouseup", "1"], env=env, check=True)
    time.sleep(0.5)
    moved = capture(env, artifacts / "dragged.png")
    assert moved == (x + 120, y + 80), f"Native drag did not follow pointer: {frame} -> {moved}"
    click(moved, env, -200, 150)
    time.sleep(0.4)
    frame = capture(env, artifacts / "dragged-shortcuts.png")
    assert frame == moved, "Opening a panel moved the dragged recorder"
    click(frame, env, 22, 0)
    time.sleep(0.2)
    click(frame, env, 22, 0)
    time.sleep(0.2)
    log.flush()
    offset = len((artifacts / "session.log").read_text())
    click(frame, env, -200, 200)
    time.sleep(0.3)
    log.flush()
    contents = (artifacts / "session.log").read_text()[offset:]
    assert ".leave(" in contents, "Transparent region kept pointer or keyboard focus"
    for index in range(8):
        click(frame, env, 22, 0)
        copier = subprocess.Popen([tool("wl-copy"), "--foreground"], env=env, stdin=subprocess.PIPE,
                                  stdout=log, stderr=log)
        processes.append(copier)
        copier.stdin.write(f"Estela isolated clipboard {index}".encode())
        copier.stdin.close()
        time.sleep(0.1)
    click(frame, env, 22, 0)
    return capture(env, artifacts / "clipboard-stress.png")


def start_wayland(env, temporary, log, processes):
    config = Path(temporary) / "weston.ini"
    shell = ""
    prefix = os.environ.get("ESTELA_TEST_PREFIX")
    if prefix:
        prefix = Path(prefix)
        backends = sorted(path for path in (prefix / "lib64").glob("libweston-*") if path.is_dir())
        backend = backends[-1] / "x11-backend.so"
        env["WESTON_MODULE_MAP"] = f"x11-backend.so={backend}"
        shell = f"--shell={prefix}/lib64/weston/desktop-shell.so"
        client = f"client={prefix}/libexec/weston-desktop-shell\n"
    else:
        client = ""
    config.write_text(f"[core]\nidle-time=0\n[shell]\n{client}panel-position=none\nlocking=false\n")
    command = [tool("weston"), "--backend=x11", "--renderer=pixman", "--width=1280",
               "--height=900", "--socket=estela-test", "--config=" + str(config)]
    if shell:
        command.append(shell)
    compositor = subprocess.Popen(command, env=env, stdout=log, stderr=log)
    processes.append(compositor)
    for _ in range(40):
        if (Path(temporary) / "estela-test").exists():
            env["WAYLAND_DISPLAY"] = "estela-test"
            return
        assert compositor.poll() is None, "Wayland compositor could not start"
        time.sleep(0.1)
    raise AssertionError("Wayland compositor did not create its socket")


def capture(env, destination):
    pixels = subprocess.check_output([tool("ffmpeg"), "-hide_banner", "-loglevel", "error",
                                      "-f", "x11grab", "-video_size", "1280x900", "-draw_mouse", "0",
                                      "-i", env["DISPLAY"], "-frames:v", "1", "-f", "rawvideo",
                                      "-pix_fmt", "rgb24", "pipe:1"], env=env)
    subprocess.run([tool("ffmpeg"), "-hide_banner", "-loglevel", "error", "-f", "rawvideo",
                    "-video_size", "1280x900", "-pixel_format", "rgb24", "-i", "pipe:0",
                    "-frames:v", "1", "-update", "1", "-y", str(destination)],
                   input=pixels, env=env, check=True)
    points = []
    for offset in range(0, len(pixels), 3):
        red, green, blue = pixels[offset:offset + 3]
        if red > 90 and 15 < green < 125 and 10 < blue < 125 and red > green * 2:
            points.append((offset // 3 % WIDTH, offset // 3 // WIDTH))
    assert points, f"Record button is missing: {destination}"
    top = min(y for _, y in points)
    points = [(x, y) for x, y in points if y < top + 48]
    bottom = max(y for _, y in points)
    assert bottom - top >= 24, f"Record button is vertically clipped: {destination}"
    return max(x for x, _ in points), (top + bottom) // 2


def click(frame, env, delta_x, delta_y):
    x, y = frame
    subprocess.run([tool("xdotool"), "mousemove", str(x + delta_x), str(y + delta_y), "click", "1"],
                   env=env, check=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Check recorder panels on an isolated Linux desktop.")
    parser.add_argument("--wayland", action="store_true")
    args = parser.parse_args()
    run(args.wayland)
