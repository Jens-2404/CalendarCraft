#!/usr/bin/env python3
"""Liest aus dem Gemeindekalender-PDF (Text) je Tag die gedruckten Zeiten.

Eingabe:  Download/Kalender_5784.pdf (nur lokal)
Ausgabe:  testdata/private/frankfurt-5784-pdf.json (von Git ausgeschlossen)

Das PDF listet die Tage spaltenweise in Folge; Tag n gehört zum Datum
Start + n (Start = 15.9.2023, Erew Rosch HaSchana).
"""
import datetime, io, json, re, sys
from pypdf import PdfReader

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")
START = datetime.date(2023, 9, 15)
WD = ("Mo", "Di", "Mi", "Do", "Fr", "Sa", "So")

lines = []
for page in PdfReader("Download/Kalender_5784.pdf").pages:
    t = re.sub(r"[֐-׿‎‏]+", "", page.extract_text())
    lines += [re.sub(r"\s+", " ", l).strip() for l in t.splitlines()]
lines = [l for l in lines if l]

records, cur = [], None
for i, l in enumerate(lines):
    nxt = lines[i + 1] if i + 1 < len(lines) else ""
    is_start = re.fullmatch(r"\d{1,2}", l) and (
        re.match(rf"^({'|'.join(WD)})\b", nxt) or False
    )
    # "23 Sa Haasinu" steht manchmal in einer Zeile
    one_line = re.match(rf"^(\d{{1,2}}) ({'|'.join(WD)})\b(.*)$", l)
    if is_start:
        cur = {"greg_day": int(l), "text": []}
        records.append(cur)
    elif one_line:
        cur = {"greg_day": int(one_line.group(1)), "text": [one_line.group(2) + one_line.group(3)]}
        records.append(cur)
    elif cur is not None:
        if l.startswith(("FB =", "Staatliche", "Westend", "Rabbinat", "gemeinde@", "www.", "T +", "F +", "rabbinat@", "2023/24", "5784", "60325", "Frankfurt am Main K.d")):
            cur = None
            continue
        cur["text"].append(l)

out = []
for n, r in enumerate(records):
    d = START + datetime.timedelta(days=n)
    text = " ".join(r["text"])
    entry = {"date": d.isoformat(), "greg_day": r["greg_day"], "text": text}
    if r["greg_day"] != d.day:
        entry["warnung"] = f"Tag {r['greg_day']} passt nicht zu {d}"
    m = re.search(r"FB (\d+[.:]\d+) FE (\d+[.:]\d+)", text)
    if m:
        entry["fb"], entry["fe"] = m.group(1), m.group(2)
        text = text.replace(m.group(0), "")
    entry["times"] = re.findall(r"(?<![\d.])(\d{1,2}[.:]\d{2})(?![\d])", text)
    out.append(entry)

json.dump(out, open("testdata/private/frankfurt-5784-pdf.json", "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print(len(out), "Tage;", sum(1 for e in out if "warnung" in e), "Warnungen")
for e in out[:5] + out[60:64]:
    print(e["date"], e["times"], e.get("fb"), "|", e["text"][:60])
