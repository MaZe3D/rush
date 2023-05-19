# rush - remote shell for microcontrollers
rush ist eine kompakte Shell für Microcontroller welche es dem Benutzer erlaubt zustände von Peripherie an Microcontrollern zu manipulieren oder auszulesen.
Die Software ist ausschließlich Kompatibel mit einem ESP32-S3 Chip.

## rush-service
Der rush-service oder auch nur rush bezeichnet den Dienst welcher auf dem Microcontroller läuft.
Um mit dem Microcontroller interagieren zu können, ist es notwendig sich mit dem WLAN-Netzwerk von diesem zu verbinden.
Es kann unter der SSID „rush“ gefunden werden.
Es existiert kein Passwort und die Verbindung ist unverschlüsselt.
Nach dem Verbinden mit dem Netzwerk ist es notwendig folgende Nezwerkeinstellungen vorzunehmen:
| IP-Adresse   | Eine Adresse aus dem Bereich `192.168.2.2` bis `192.168.2.254`                           |
| ------------ | ---------------------------------------------------------------------------------------- |
| Subnetzmaske | `255.255.255.0` oder in CIDR `192.168.2.2/24`                                            |
| Gateway      | Keines Eintragen! Kann zum absturz des Systems führen. Sollte die Eingabe unvermeidbar sein, bitte diese auf 0.0.0.0 setzen. |
| Nameserver   | Sollte die Eingabe eines Nameservers unvermeidbar sein, bitte diesen auf 0.0.0.0 setzen. |

Nun lässt sich eine TCP-Verbindung über einen TCP-Client aufbauen (IP: 192.168.2.1; Port: 2000).
Wir empfehlen [rush-client](#rush-client), Netcat funktioniert aber auch.

## rush-client
Der rush-Client benötigt eine IP-Adresse und einen Port als Ziel des Verbindungsaufbaus.
Diese Parameter werden als Kommandozeilenargument übergeben.
Unter Windows sehen Programmaufrufe zum Beispiel so aus:
```
.\rush_client.exe 192.168.2.1:2000

.\rush_client.exe 127.0.0.1:8080
```
Der zweite Befehl dient dazu, eine Verbindung mit dem localhost aufzubauen.
Dies ermöglicht es, mit einem Programm wie Netcat, den rush-Client unabhängig von einem Mikrocontroller zu testen.


## Befehlssatz
Wenn die Verbindung zum rush-service über einen TCP-Client steht, kann mit dem Microcontroller Interagiert werden.
Unsere Software verwendet einen definierten Befehlssatz zur Manipulation und Beobachtung von Peripherie.
| Befehl                   | Beschreibung                                                                 | Rückgabewert                                                                        |
| ------------------------ | ---------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| `read gpio.[a]`          | liest den aktuellen Zustand des GPIO-Pins mit der Nummer a aus               | `gpio.[a]=[0\|1]` wobei `0` = LOW-Pegel und `1` = HIGH-Pegel                        |
| `write gpio.[a] [value]` | setzt den Zustand des GPIO-Pins mit der Nummer a auf [value] (siehe unten)   | n/a                                                                                 |
| `watch gpio.[a]`         | abonniert jede Zustandsänderung des GPIO-Pins mit der Nummer a               | neuer Zustand des Pins, Syntax wie bei `read`                                       |
| `unwatch gpio.[a]`       | beendet das Abonnement der Zustandsänderungen des GPIO-Pins mit der Nummer a | n/a                                                                                 |

`a` gibt die Nummer eines GPIO-Pins an. Gültig ist eine Zahl von 0 bis 48 (einschließlich), wobei die GPIO-Pins mit den Nummern 22, 23, 24 und 25 nicht vorhanden sind.

`value` kann einen HIGH- oder LOW-Pegel über verschiedene Begriffe annehmen.
| LOW-Pegel | HIGH-Pegel |
| --------- | ---------- |
| `low`     | `high`     |
| `l`       | `h`        |
| `off`     | `on`       |
| `0`       | `1`        |
| `false`   | `true`     |
| `f`       | `t`        |
