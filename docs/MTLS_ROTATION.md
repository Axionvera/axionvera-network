**mTLS CA Rotation**
- **Purpose**: Explain how to rotate the client CA used to validate mTLS client certificates for validator nodes.

- **Preferred workflow**:
  - Host the new CA PEM at a secure URL (internal storage for operator use).
  - Run `scripts/rotate_ca.sh <CA_URL> /etc/axionvera/mtls/clients_ca.pem "systemctl restart axionvera-node"` on the node, or use the PowerShell script on Windows.

- **Atomic update**: The scripts replace the file atomically to avoid partial reads by the server.

- **Automatic restart**: Provide a restart command to the script to trigger a service restart (or use `systemctl reload`/SIGHUP if supported). If you run inside containers, use `docker restart <container>` or restart the pod.

- **Notes about runtime reload**:
  - The current `GrpcServer` config loads TLS settings at startup. After rotation the service must be restarted to pick up the new CA bundle, or your orchestration can replace the pod/container.
  - Implementation of dynamic in-process TLS reloading is possible but more invasive; open an issue/PR if you want an in-process hot-reload.
