# harmonia-project

## Compilation et exécution

Ce projet est développé en Rust. Pour le compiler et l'exécuter, vous devez d'abord vous placer dans le dossier `harmonia` :

```bash
cd harmonia
```

### Compiler

Pour compiler le projet (mode développement) :

```bash
cargo build
```

Pour compiler le projet avec les optimisations (mode release) :

```bash
cargo build --release
```

### Exécuter

Pour compiler et lancer automatiquement l'application :

```bash
cargo run
```

Pour lancer la version optimisée :

```bash
cargo run --release
```

### Exécutable en mode debug

Un exécutable en mode debug est déjà compilé et disponible. Vous pouvez le retrouver et l'exécuter à l'emplacement suivant :

```bash
./target/debug/debug_mod
```

*(Chemin complet depuis la racine du projet : `harmonia/target/debug/debug_mod`)*
