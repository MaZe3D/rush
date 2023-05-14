# rush - remote shell for microcontrollers
rush ist eine kompakte Shell für Microcontroller welche es dem Benutzer erlaubt zustände von Peripherie an Microcontrollern zu manipulieren oder auszulesen. Die Software ist ausschließlich Kompatibel mit einem ESP32-S3 Chip.

## rush-service
Der rush-service oder auch nur rush bezeichnet den Dienst welcher auf dem Microcontroller läuft.
Um mit dem Microconroller interagieren zu können, ist es notwendig sich mit dem WLAN-Netzwerk von diesem zu verbinden. Es kann unter der SSID „rush“ gefunden werden. Es existiert kein Passwort und die Verbindung ist unverschlüsselt.
Nach dem Verbinden mit dem Netzwerk ist es notwendig folgende Nezwerkeinstellungen vorzunehmen:
| IP-Adresse   | Eine Adresse aus dem Bereich `192.168.2.2` bis `192.168.2.254`                           |
| ------------ | ---------------------------------------------------------------------------------------- |
| Subnetzmaske | `255.255.255.0` oder in CIDR `192.168.2.2/254`                                           |
| Gateway      | 192.168.2.1                                                                              |
| Nameserver   | Sollte die Eingabe eines Nameservers unvermeidbar sein, bitte diesen auf 0.0.0.0 setzen. |

Nun lässt sich eine TCP-Verbindung über einen TCP-Client aufbauen. Wir empfehlen [rush-client](#rush-client).

## rush-client
Der rush-Client benötigt eine IP-Adresse, zu der er die Verbindung aufbauen soll. Diese wird als Kommandozeilenargument übergeben. Der Programmaufruf sieht dann bei-spielhaft in Windows wie folgt aus. 
```
.\rush_client.exe 192.168.2.1:80

.\rush_client.exe 127.0.0.1:8080
```
Der zweite Befehl dient dazu, eine Verbindung mit dem localhost aufzubauen. Dies ermöglicht es, mit einem Programm wie ncat, den rush-Client unabhängig von einem Mikrocontroller zu testen. 


## Befehlssatz
Wenn die Verbindung zum rush-service über einen TCP-Client steht, kann mit dem Microcontroller Interagiert werden. Unsere Software verwendet einen definierten Befehlssatz zur Manipulation und Beobachtung von Peripherie.
| Befehl                   | Beschreibung                                                                 | Rückgabewert                                                                        |
| ------------------------ | ---------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| `read gpio.[a]`          | Liest den aktuellen Zustand   eines GPIO-Pins mit der Nummer a aus           | `gpio.[a]=[0\|1]` wobei `0` = LOW-Pegel und `1` = HIGH-Pegel                        |
| `write gpio.[a] [value]` | Manipuliert den   Zustand des GPIO-Pins mit der Nummer a.                    | n/a                                                                                 |
| `watch gpio.[a]`         | Abonniert jede Zustandsänderung des GPIO-Pins mit der Nummer a.              | Gibt unaufgefordert den aktuellen Zustand nach folgender Syntax:  `gpio.[a]=[0\|1]` |
| `unwatch gpio.[a]`       | Beendet das   Abonnement zu Zustandsänderungen am GPIO-Pin mit der Nummer a. | n/a                                                                                 |

`a` gibt die Nummer eines GPIO-Pins an. Gültig ist eine Zahl zwischen 0 und 48. Wobei die GPIO-Pins mit der Nummer 22, 23, 24 und 25 nicht vorhanden sind.

`value` kann einen HIGH- oder LOW-Pegel über verschiedene Begriffe annehmen.
| LOW-Pegel | HIGH-Pegel |
| --------- | ---------- |
| `low`     | `high`     |
| `l`       | `h`        |
| `off`     | `on`       |
| `0`       | `1`        |
| `false`   | `true`     |
| `f`       | `t`        |
