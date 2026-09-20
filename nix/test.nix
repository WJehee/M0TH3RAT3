# VM test: boots a machine with the module enabled, logs in over SSH with a
# password from the seeded save file, explores a planet, and checks that the
# progress was written back to the state directory.
{ self, testers, writeText, ... }:
let
    ship = writeText "ship.json" (builtins.toJSON {
        path = "seed.json";
        users = [{
            username = "captain";
            password_start = "";
            password_attempts = 0;
            password_attempts_max = 0;
            password = "hunter2";
            pos_x = 5.0;
            pos_y = 5.0;
            fuel = 10;
            crystals = 3;
            reputation = 1;
        }];
        components = 0;
        map = [{
            name = "Sol";
            pos = [ 5.0 5.0 ];
            planets = [{
                name = "Terra";
                x = 30.0;
                y = 50.0;
                radius = 5.0;
                planet_type = "Terrestrial";
                has_event = false;
                has_component = true;
                crystals = 2;
                fuel = 1;
                visited_by = [ ];
            }];
        }];
    });

    # Drives ssh through a pty the way a player would: answer the password
    # prompt, open the star map, explore the planet, quit with Esc.
    play = writeText "play.py" ''
        import fcntl, os, pty, select, struct, sys, termios, time

        pid, fd = pty.fork()
        if pid == 0:
            os.environ["TERM"] = "xterm-256color"
            os.execvp("ssh", ["ssh", "-tt", "-p", "2222",
                "-o", "StrictHostKeyChecking=no", "-o", "UserKnownHostsFile=/dev/null",
                "captain@localhost"])

        # A pty starts with no size; ssh would report 0x0 to the server.
        fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 50, 120, 0, 0))

        buf = b""
        def read_for(seconds):
            global buf
            end = time.time() + seconds
            while time.time() < end:
                r, _, _ = select.select([fd], [], [], 0.1)
                if fd in r:
                    try:
                        data = os.read(fd, 65536)
                    except OSError:
                        return False
                    if not data:
                        return False
                    buf += data
            return True

        end = time.time() + 20
        while b"assword" not in buf and time.time() < end:
            read_for(0.5)
        assert b"assword" in buf, buf
        os.write(fd, b"hunter2\n")
        buf = b""
        read_for(4)
        assert b"assword" not in buf, "login rejected"
        os.write(fd, b"\x1b[B"); read_for(0.5)
        os.write(fd, b"\r"); read_for(1)
        os.write(fd, b"e"); read_for(1)
        assert b"Zonnestelsels" in buf, buf[-500:]
        os.write(fd, b"\x1b")
        alive = read_for(5)
        assert not alive, "server did not close the session"
        _, status = os.waitpid(pid, 0)
        assert os.waitstatus_to_exitcode(status) == 0, status
        print("ok")
    '';
in testers.nixosTest {
    name = "mothership-service";

    nodes.machine = { pkgs, ... }: {
        imports = [ self.nixosModules.mothership ];
        services.mothership = {
            enable = true;
            initialStorage = ship;
            listenAddress = "127.0.0.1";
        };
        environment.systemPackages = [ pkgs.openssh pkgs.python3 ];
    };

    testScript = ''
        machine.wait_for_unit("mothership.service")
        machine.wait_for_open_port(2222)
        machine.succeed("test -s /var/lib/mothership/host_key")
        machine.succeed("timeout 60 python3 ${play}")
        machine.wait_until_succeeds("journalctl -u mothership | grep -q 'Session ended'")
        # Exploring Terra hands out its crystals, fuel and component.
        machine.succeed("grep -q '\"visited_by\": \\[' /var/lib/mothership/ship.json")
        machine.succeed("grep -q '\"components\": 1' /var/lib/mothership/ship.json")
        machine.succeed("grep -q '\"fuel\": 11' /var/lib/mothership/ship.json")
        # The host key must survive a restart so clients keep trusting it.
        key = machine.succeed("cat /var/lib/mothership/host_key")
        machine.succeed("systemctl restart mothership.service")
        machine.wait_for_open_port(2222)
        assert key == machine.succeed("cat /var/lib/mothership/host_key")
    '';
}
