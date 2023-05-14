# rush - remote shell for microcontrollers
rush ist eine kompakte Shell für Microcontroller welche es dem Benutzer erlaubt zustände von Peripherie an Microcontrollern zu manipulieren oder auszulesen.

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

## Befehlssatz
Wenn die Verbindung zum rush-service über einen TCP-Client steht, kann mit dem Microcontroller Interagiert werden. Unsere Software verwendet einen definierten Befehlssatz zur Manipulation und Beobachtung von Peripherie.
| Befehl                   | Beschreibung                                                                 | Rückgabewert                                                                        |
| ------------------------ | ---------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| `read gpio.[a]`          | Liest den aktuellen Zustand   eines GPIO-Pins mit der Nummer a aus           | `gpio.[a]=[0\|1]` wobei `0` = LOW-Pegel und `1` = HIGH-Pegel                        |
| `write gpio.[a] [value]` | Manipuliert den   Zustand des GPIO-Pins mit der Nummer a.                    | n/a                                                                                 |
| `watch gpio.[a]`         | Abonniert jede Zustandsänderung des GPIO-Pins mit der Nummer a.              | Gibt unaufgefordert den aktuellen Zustand nach folgender Syntax:  `gpio.[a]=[0\|1]` |
| `unwatch gpio.[a]`       | Beendet das   Abonnement zu Zustandsänderungen am GPIO-Pin mit der Nummer a. | n/a                                                                                 |

`a` gibt die Nummer eines GPIO-PIns an. Gültig ist eine Zahl zwischen 0 und 48.