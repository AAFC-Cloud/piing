<div align="center">
    <h1>📡 Piing</h1>
    <br/>

[See English version](./README.md)

</div>

## Description

Un utilitaire de ping HTTP et TCP moderne écrit en Rust. Piing fournit plusieurs méthodes de mesure pour tester avec précision la connectivité réseau et la latence lorsque les paquets ICMP ne sont pas disponibles.

## Fonctionnalités

- **Mode de connexion TCP** : Mesure de type ping la plus précise utilisant des connexions TCP brutes
- **Requêtes HTTP GET/HEAD** : Tests de connectivité basés sur HTTP traditionnels
- **Sortie colorisée** : Rétroaction visuelle avec des temps de réponse codés par couleur
- **Chronométrage flexible** : Analyse d'intervalles lisibles par l'humain (par ex., "1s", "500ms", "2.5s")
- **Protocoles multiples** : Support pour HTTP et HTTPS avec détection automatique

## Installation

### Prérequis

- [Rust](https://rustup.rs/) (dernière version stable)

### Compilation à partir des sources

```powershell
git clone https://github.com/your-username/piing.git
cd piing
cargo build --release
```

L'exécutable sera disponible à `target/release/piing.exe`.

## Utilisation

### Ping HTTP de base
```powershell
piing google.com
```

### Ping de connexion TCP (plus précis)
```powershell
piing google.com --tcp
```

### Requêtes HTTP HEAD (plus rapide que GET)
```powershell
piing google.com --head
```

### Intervalle personnalisé
```powershell
piing google.com --interval 500ms
```

### Port personnalisé pour ping TCP
```powershell
piing google.com --tcp --port 443
```

### Exemple complet
```powershell
piing https://example.com --tcp --port 443 --interval 2s
```

## Options de ligne de commande

| Option | Court | Description |
|--------|-------|-------------|
| `--tcp` | | Utiliser la connexion TCP pour une mesure de type ping la plus précise |
| `--head` | | Utiliser HTTP HEAD au lieu de GET (pas de corps de réponse) |
| `--interval` | `-i` | Intervalle de rafraîchissement (par ex., "1s", "500ms", "2.5s") |
| `--port` | `-p` | Port à utiliser pour le ping TCP (défaut : 80 pour HTTP, 443 pour HTTPS) |
| `--help` | `-h` | Afficher les informations d'aide |

## Sortie

L'utilitaire affiche les résultats horodatés avec des temps de réponse codés par couleur :

- **Vert** : Temps de réponse < 100ms
- **Jaune** : Temps de réponse 100-500ms  
- **Rouge** : Temps de réponse > 500ms

### Exemple de sortie

```
TCP pinging google.com:443 every 1s

Thu, 12 Jun 2025 08:48:10 -0400 - TCP Connect: SUCCESS - Duration: 29.2ms
Thu, 12 Jun 2025 08:48:11 -0400 - TCP Connect: SUCCESS - Duration: 28.3ms
Thu, 12 Jun 2025 08:48:12 -0400 - TCP Connect: SUCCESS - Duration: 33.5ms
```

## Précision des mesures

### Mode de connexion TCP (recommandé)
- **Le plus précis** pour les mesures de type ping
- Mesure seulement le temps de réseau + poignée de main TCP
- Exclut les frais généraux HTTP/TLS et le traitement serveur
- Équivalent le plus proche du ping ICMP lorsque ICMP n'est pas disponible

### Mode HTTP HEAD
- Plus précis que les requêtes GET
- Inclut la poignée de main TLS mais pas le téléchargement du corps de réponse
- Bon équilibre entre précision et conformité au protocole

### Mode HTTP GET
- Cycle complet de requête/réponse HTTP
- Inclut tous les frais généraux de réseau, TLS, HTTP et traitement serveur
- Utile pour tester la pile d'applications complète
