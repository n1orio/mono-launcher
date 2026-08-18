# Änderungen

## [1.3.0] — 2026-08-18

### Bibliothek
- Neuer Tab **„Bibliothek"**: alle installierten Modpacks, nach Kategorien gruppiert, Kacheln mit Zoom.
- Jede Kachel zeigt den aktiven Modloader/die Version oder „nicht installiert"; Kategorien lassen sich ausblenden.
- Neuer Abschnitt **„Zuletzt"** in der Seitenleiste zeigt Ihre zuletzt gestarteten Modpacks (bis zu 10).

### Modpack-Branding
- **Modpack umbenennen** direkt in der Oberfläche — benutzerdefinierte Namen für eingebaute Modpacks werden pro Instanz gespeichert, bei benutzerdefinierten wird `packs.json` aktualisiert.
- **Eigenes Banner hochladen** (Icon-Bearbeitung gab es bereits) — wird lokal pro Modpack gespeichert.

### Mods & Updates
- Manuell installierte Mods zeigen jetzt echte Metadaten — Icon von Modrinth/CurseForge, Projekttitel und Version — statt roher Dateinamen.
- **„Alle aktualisieren" ist jetzt tabweise** (Mods / Ressourcenpakete / Shader), mit Zähler.
- Die Modrinth-Suche bietet einen **Versionsart-Filter** (Release/Beta/Alpha), Loader-Filter, „Mehr laden" und „Filter zurücksetzen".
- Verwaiste Einträge getrackter Mods (Dateien bereits von der Festplatte entfernt) werden bereinigt, sodass das Update-Abzeichen keine Phantom-Updates mehr zeigt.

### Gameplay & UI
- Während das Spiel läuft, wird aus dem „Spielen"-Button ein roter **„Stoppen"**-Button, der den Prozess beendet — mit sauberem Exit (kein falscher Absturz-Bericht bei beabsichtigtem Stopp).
- Der Hauptbereich der Oberfläche ist jetzt **zentriert** statt randlos gestreckt.

### Unter der Haube
- Neuer persistenter **HTTP-Cache** (bedingte Anforderungen ETag/304), sodass wiederholte Launcher-Starts keine unveränderten Daten erneut laden.
- Erweiterte Rust-Unit-Tests; Typprüfung des Frontends.