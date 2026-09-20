# NixOS module that runs the Mothership interface as an SSH server.
#
# Usage in a flake-based configuration:
#
#   inputs.mothership.url = "github:wjehee/M0TH3RAT3";
#   ...
#   imports = [ inputs.mothership.nixosModules.mothership ];
#   services.mothership = {
#     enable = true;
#     port = 2222;
#     openFirewall = true;
#     initialStorage = ./ship.json;
#   };
#
# Players then connect with `ssh -p 2222 <username>@<host>`. The service does
# not use the system sshd: the binary embeds its own SSH server, so it can
# share a machine with a normal sshd on port 22.
{ self }:
{ config, lib, pkgs, ... }:
let
    cfg = config.services.mothership;
    stateDirectory = "mothership";
    dataDir = "/var/lib/${stateDirectory}";
    storageFile = "${dataDir}/ship.json";
    hostKey = "${dataDir}/host_key";
in {
    options.services.mothership = {
        enable = lib.mkEnableOption "the Mothership ship interface served over SSH";

        package = lib.mkOption {
            type = lib.types.package;
            default = self.packages.${pkgs.stdenv.hostPlatform.system}.mothership;
            defaultText = lib.literalExpression "mothership.packages.\${system}.mothership";
            description = "The mothership package to run.";
        };

        listenAddress = lib.mkOption {
            type = lib.types.str;
            default = "0.0.0.0";
            description = "Address to bind the SSH server to.";
        };

        port = lib.mkOption {
            type = lib.types.port;
            default = 2222;
            description = "TCP port for the embedded SSH server.";
        };

        openFirewall = lib.mkOption {
            type = lib.types.bool;
            default = false;
            description = "Open the port in the firewall.";
        };

        initialStorage = lib.mkOption {
            type = lib.types.nullOr lib.types.path;
            default = null;
            description = ''
                Save file to install on first start. Copied to
                ${storageFile} only when that file does not exist yet, so
                the players' progress survives redeploys. Without it the
                service starts with an empty galaxy and no users, which
                means nobody can log in.
            '';
        };

        logLevel = lib.mkOption {
            type = lib.types.str;
            default = "info";
            description = "RUST_LOG filter for the service.";
        };
    };

    config = lib.mkIf cfg.enable {
        systemd.services.mothership = {
            description = "Mothership ship interface over SSH";
            wantedBy = [ "multi-user.target" ];
            after = [ "network.target" ];

            preStart = ''
                if [ ! -e ${storageFile} ]; then
                    ${lib.optionalString (cfg.initialStorage != null) ''
                        install -m 0600 ${cfg.initialStorage} ${storageFile}
                    ''}
                    ${lib.optionalString (cfg.initialStorage == null) ''
                        echo "no save file at ${storageFile} and no initialStorage set; starting empty" >&2
                        install -m 0600 /dev/stdin ${storageFile} <<'JSON'
                        {"path": "${storageFile}", "users": [], "components": 0, "map": []}
                        JSON
                    ''}
                fi
            '';

            environment.RUST_LOG = cfg.logLevel;

            serviceConfig = {
                ExecStart = lib.escapeShellArgs [
                    "${cfg.package}/bin/mothership"
                    "--serve"
                    "--listen" "${cfg.listenAddress}:${toString cfg.port}"
                    "--host-key" hostKey
                    storageFile
                ];
                WorkingDirectory = dataDir;
                StateDirectory = stateDirectory;
                StateDirectoryMode = "0700";
                DynamicUser = true;
                Restart = "on-failure";
                RestartSec = 5;

                # The process only needs the network and its state directory.
                AmbientCapabilities = lib.optional (cfg.port < 1024) "CAP_NET_BIND_SERVICE";
                CapabilityBoundingSet = lib.optional (cfg.port < 1024) "CAP_NET_BIND_SERVICE";
                NoNewPrivileges = true;
                PrivateTmp = true;
                PrivateDevices = true;
                ProtectSystem = "strict";
                ProtectHome = true;
                ProtectKernelTunables = true;
                ProtectKernelModules = true;
                ProtectControlGroups = true;
                ProtectClock = true;
                ProtectHostname = true;
                ProtectProc = "invisible";
                RestrictAddressFamilies = [ "AF_INET" "AF_INET6" ];
                RestrictNamespaces = true;
                RestrictRealtime = true;
                RestrictSUIDSGID = true;
                LockPersonality = true;
                MemoryDenyWriteExecute = true;
                SystemCallArchitectures = "native";
                SystemCallFilter = [ "@system-service" "~@privileged" ];
                UMask = "0077";
            };
        };

        networking.firewall.allowedTCPPorts = lib.mkIf cfg.openFirewall [ cfg.port ];
    };
}
