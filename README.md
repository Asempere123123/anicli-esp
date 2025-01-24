# Ver Anime desde el terminal
Permite ver anime desde el terminal con una interfaz simple e intuitiva. Funciona en todos los sistemas operativos sin dependencias (en WSL se necesita WSLU `sudo apt install wslu`).

!["Ejemplo de uso](./example.gif)
```bash
anicli-esp
```

# Instalación
## Cargo
Se puede instalar desde cargo
```bash
cargo install anicli-esp
```

# Uso
## Conifgurar
La primera vez que se abre permite elegir que aplicación se utilizará para abrir el video. Esto se puede cambiar mas adelante abriendolo con el argumento --config.
```bash
anicli-esp -c
anicli-esp --config
```
## Interfaz
Con el tabulador y shift + tabulador se puede cambiar la ventana seleccionada. El enter permite selecionar una opción.
