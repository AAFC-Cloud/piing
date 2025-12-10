<div align="center">
    <h1>📡 Piing</h1>
    <br/>

[Voir la version anglaise](./README.md)

</div>

## Description

Un utilitaire de ping continu qui réside dans la zone de notification (system tray) de Windows.

Il effectue des mesures dans le temps afin d'identifier les tendances de performance du réseau et les incidents de panne, tout en gardant une visibilité sur la connectivité VPN comme facteur possible de requêtes échouées.

## Galerie

L'application ajoute une icône dans la zone de notification

<img src="media/system_tray.png" alt="Icône de Piing dans la zone de notification"/>

La console est masquée par défaut, mais peut être ouverte pour voir les journaux en temps réel

<img src="media/console.png" alt="Journaux de la console Piing"/>

Le répertoire maison stocke la configuration, les journaux et les critères VPN dans des formats simples et ouverts

<img src="media/home.png" alt="Répertoire maison de Piing"/>

## Sortie

Par défaut, l'application écrit des fichiers journaux au format JSON délimité par des sauts de ligne (ndjson) dans `$PIING_HOME/logs/`. Chaque résultat de ping est consigné avec un horodatage, l'hôte, le mode, l'état de réussite, la latence et le contexte VPN.

```json
{"timestamp":"2025-12-02T04:17:28.879441Z","level":"INFO","fields":{"message":"Ping succeeded","host":"8.8.8.8","mode":"icmp","success":true,"latency_ms":23.2756}}
{"timestamp":"2025-12-02T04:17:29.909676Z","level":"INFO","fields":{"message":"Ping succeeded","host":"8.8.8.8","mode":"icmp","success":true,"latency_ms":22.2433}}
{"timestamp":"2025-12-02T04:17:30.935951Z","level":"INFO","fields":{"message":"Ping succeeded","host":"8.8.8.8","mode":"icmp","success":true,"latency_ms":24.1527}}
```

## Configuration

### Mode

Piing prend en charge plusieurs modes de ping :
- `icmp` : Requêtes ICMP classiques (nécessitent des privilèges élevés sur certains systèmes)
- `tcp` : Paquets TCP SYN vers le port 80/443
- `http-head` : Requêtes HTTP HEAD vers l'hôte
- `http-get` : Requêtes HTTP GET vers l'hôte

### Détection de VPN

Piing inclut une détection des adaptateurs VPN basée sur une configuration HCL pour identifier automatiquement quand des connexions VPN sont actives, ce qui ajoute un contexte aux données de performance de ping.

## Utilisation

```text
❯ piing --help
TeamDman's Windows tray ping utility

Usage: piing.exe [OPTIONS] [COMMAND]

Commands:
    run       Launch the tray application and ping monitors
    host      Manage the list of hosts to ping
    mode      Configure ping mode
    interval  Configure ping interval
    audit     Audit log files
    vpn       Manage VPN related commands
    help      Print this message or the help of the given subcommand(s)

Options:
            --debug            Enable verbose debug logging
            --log-file <FILE>  Write structured ndjson logs to this file instead of the default in `$PIING_HOME/logs`
    -h, --help             Print help
    -V, --version          Print version
```

Arborescence complète des commandes :

```text
piing help # Show help
piing run # Start the tray application, default behaviour when no arguments
piing host [add|remove|list] # Manage ping hosts
piing mode [set|get] # Configure ping mode
piing interval [set|get] # Configure ping interval
piing audit # Audit ping log files
piing vpn [check|adapter [add|remove|list|get-path]] # Manage VPN related commands
```

## Droits d’auteur

Les droits d’auteur appartiennent à © Sa Majesté le Roi du chef du Canada, représenté par le ministre de l’Agriculture et de l’Agroalimentaire, 2025.