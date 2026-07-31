# BlackMamba Recall

Agente local de autocompletado global y aprendizaje supervisado de procesos para macOS, navegador y terminal.

> Estado: MVP seguro y offline. Registra, persiste y simula workflows; todavía no controla SoundCloud ni publica contenido real.

## Visión

Recall aprende procesos completos paso a paso. El usuario realiza primero el flujo; después el agente sugiere acciones conocidas y, tras varias ejecuciones correctas, puede automatizar únicamente pasos seguros y de alta confianza.

Cuando la página cambia, aparece un error o la confianza baja, Recall se detiene, entra en modo `exception` y devuelve el control al usuario. La acción final de publicar siempre requiere confirmación humana.

## Modos

```text
observation -> assisted -> automatic
      ^             |           |
      |             |           v
      +-------------+------ exception

cualquier modo <-> paused
```

- `observation`: registra lo que hace el usuario.
- `assisted`: propone el siguiente paso, pero el usuario confirma o corrige.
- `automatic`: ejecuta pasos conocidos por encima del umbral de confianza.
- `exception`: detiene el workflow ante errores o estados desconocidos.
- `paused`: suspende inmediatamente la observación y ejecución.

## Reglas de seguridad

- Datos locales en SQLite.
- Secretos, contraseñas, tokens y pagos se reemplazan con `[REDACTED]` antes de guardarse.
- Tres sesiones correctas para entrar en automático por defecto.
- Confianza mínima de `0.90` para ejecutar un paso automático.
- `publish`, `release`, `submit`, `delete` y `purchase` siempre requieren una persona.
- Sin telemetría ni sincronización cloud.
- Sin captura de pantalla ni hooks globales activos en este milestone.

## Arquitectura

```text
crates/core                 Modelos canónicos Workflow / Session / Step
crates/observer             Eventos observados y pausa inmediata
crates/workflow-engine      Máquina de estados y decisiones seguras
crates/storage              Persistencia SQLite auditable
crates/macos-accessibility  Frontera para Accessibility API de macOS
crates/cli                  CLI de inspección y simulación
apps/desktop                Shell visible del estado del agente
docs                        Arquitectura y decisiones
examples                    Fixtures reproducibles
```

Consulta [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) para el flujo completo.

## Requisitos

- Rust 1.82 o superior.
- macOS para la futura integración real con Accessibility API.
- SQLite se compila embebido mediante `rusqlite`.

## Ejecutar

```bash
git clone https://github.com/Blackmvmba88/blackmamba-recall.git
cd blackmamba-recall
cargo test --workspace
```

Crear la base local:

```bash
cargo run -p recall-cli -- init-db --path recall.db
```

Ver la promoción de modos:

```bash
cargo run -p recall-cli -- state-demo
```

Simular una carga de SoundCloud sin abrir ni modificar el navegador:

```bash
cargo run -p recall-cli -- demo-soundcloud --path recall.db
```

La salida debe terminar con:

```text
Final action decision: RequireUserConfirmation
No browser action was performed.
```

## Desarrollo

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Roadmap

### Milestone 1 — Base segura

- [x] Cargo workspace.
- [x] Workflow, Session y Step serializables.
- [x] Máquina de estados supervisada.
- [x] Persistencia SQLite.
- [x] Redacción de campos sensibles.
- [x] CLI y simulación SoundCloud.
- [x] Pruebas y CI en macOS.

### Milestone 2 — Observación real en macOS

- [ ] Solicitud visible de permiso Accessibility.
- [ ] Detección del campo editable enfocado.
- [ ] Captura global de `ArrowUp`, `ArrowDown`, `Enter` y `Escape`.
- [ ] Popup de opciones junto al cursor sin perder foco.
- [ ] Interruptor de emergencia fuera del proceso principal.

### Milestone 3 — Aprendizaje de SoundCloud

- [ ] Grabación paso a paso con selectores robustos.
- [ ] Comparación de estado esperado contra estado observado.
- [ ] Revisión, aprobación, corrección y borrado de reglas.
- [ ] Ejecución asistida antes de cualquier automatización.
- [ ] Publicación final permanentemente manual por defecto.

## Privacidad y seguridad

Lee [`PRIVACY.md`](PRIVACY.md) y [`SECURITY.md`](SECURITY.md) antes de activar captura o automatización real.
