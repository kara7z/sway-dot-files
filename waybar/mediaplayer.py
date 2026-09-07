#!/usr/bin/env python3
import subprocess
import json
import sys
import time

def get_mpris_metadata():
    try:
        result = subprocess.run(
            ["playerctl", "metadata", "--format", "{{playerName}}\n{{status}}\n{{artist}}\n{{title}}\n{{album}}"],
            capture_output=True, text=True, timeout=2
        )
        if result.returncode != 0:
            return None
        lines = result.stdout.strip().split("\n")
        if len(lines) < 3:
            return None
        player_name = lines[0]
        status = lines[1]
        artist = lines[2] if len(lines) > 2 else ""
        title = lines[3] if len(lines) > 3 else ""
        album = lines[4] if len(lines) > 4 else ""
        return {
            "player-name": player_name,
            "status": status,
            "artist": artist,
            "title": title,
            "album": album,
            "text": f"{artist} - {title}" if artist else title
        }
    except (subprocess.TimeoutExpired, FileNotFoundError):
        return None

def main():
    while True:
        metadata = get_mpris_metadata()
        if metadata is None:
            print("", flush=True)
        else:
            print(json.dumps(metadata), flush=True)
        sys.stdout.flush()
        time.sleep(1)

if __name__ == "__main__":
    main()
