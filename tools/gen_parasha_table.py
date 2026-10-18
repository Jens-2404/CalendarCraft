#!/usr/bin/env python3
"""Erzeugt crates/cc-holidays/src/parasha_table.rs aus Hebcal-Jahresdaten.

Die Wochenabschnitte (Paraschot) hängen nur vom Jahrestyp ab: Wochentag
von Rosch HaSchana und Länge des Jahres (14 Typen, "Keviyot"). Dieses
Skript liest für viele Jahre die Hebcal-Ausgabe (Diaspora), prüft, dass
alle Jahre desselben Typs dieselbe Wochenfolge haben, und schreibt je Typ
die Tabelle: (Schabbat-Nummer ab dem ersten Schabbat auf/nach Rosch
HaSchana, erster Abschnitt, ist Doppelabschnitt).

Aufruf (aus dem Projektordner):
    python tools/gen_parasha_table.py

Eingabe: testdata/private/years/h<Jahr>.json (von Git ausgeschlossen),
geholt mit
    https://www.hebcal.com/hebcal?v=1&cfg=json&year=<Jahr>&yt=H&s=on&i=off&lg=a
"""
import collections
import datetime
import glob
import io
import json
import re
import sys

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8")
BASE = "testdata/private/years/"
OUT = "crates/cc-holidays/src/parasha_table.rs"

PAIRS = {
    "Vayakhel-Pekudei": 21,
    "Tazria-Metzora": 26,
    "Achrei Mos-Kedoshim": 28,
    "Behar-Bechukosai": 31,
    "Chukas-Balak": 38,
    "Matos-Masei": 41,
    "Nitzavim-Vayeilech": 50,
}
HAAZINU = 52


def items(year):
    with open(f"{BASE}h{year}.json", encoding="utf-8") as f:
        return json.load(f)["items"]


def rosh_hashana(year):
    for i in items(year):
        t = i["title"]
        if i["category"] == "holiday" and t.startswith("Rosh Hashana") and "II" not in t:
            return datetime.date.fromisoformat(i["date"])
    raise SystemExit(f"Rosch HaSchana {year} nicht gefunden")


years = sorted(int(re.search(r"h(\d+)", f).group(1)) for f in glob.glob(BASE + "h*.json"))
types = collections.defaultdict(list)
for y in years:
    if y + 1 not in years:
        continue
    rh, rh2 = rosh_hashana(y), rosh_hashana(y + 1)
    length = (rh2 - rh).days
    wd = (rh.weekday() + 1) % 7  # 0 = Sonntag
    first_sat = rh + datetime.timedelta(days=(6 - wd) % 7)
    seq = []
    nxt = HAAZINU
    for i in items(y):
        if i["category"] != "parashat":
            continue
        d = datetime.date.fromisoformat(i["date"])
        if d >= rh2:
            continue
        name = i["title"].replace("Parshas ", "").replace("Parashat ", "")
        week = (d - first_sat).days // 7
        if name in PAIRS:
            first, pair = PAIRS[name], True
        else:
            first, pair = None, False
        if first is None:
            # Einzelabschnitt: Das Jahr beginnt mit Haazinu, bei Rosch HaSchana
            # an Montag/Dienstag zusätzlich mit Vajelech davor (Schabbat Schuwa).
            if name == "Vayeilech":
                first = 51
            elif name.startswith("Ha"):
                first = HAAZINU
            else:
                first = nxt
        seq.append((week, first, pair))
        nxt = first + (2 if pair else 1)
        if nxt == HAAZINU + 1:
            nxt = 0  # nach Haazinu beginnt der Zyklus bei Bereschit
    types[(wd, length)].append((y, seq))

lines = []
for key in sorted(types):
    ref = types[key][0][1]
    for y, seq in types[key]:
        if seq != ref:
            raise SystemExit(f"Inkonsistent: Typ {key}, Jahre {types[key][0][0]} und {y}")
    # Plausibilität: Die Folge endet mit Nitzavim (50) oder Nitzavim-Vajelech (50-51)
    last_week, last_first, last_pair = ref[-1]
    if last_first != 50:
        raise SystemExit(f"Typ {key} endet nicht bei Nitzavim: {ref[-1]}")
    body = ", ".join(f"({w}, {f}, {'true' if p else 'false'})" for w, f, p in ref)
    lines.append(
        f"    YearType {{ rh_weekday: {key[0]}, length: {key[1]}, readings: &[{body}] }},"
    )
    print(key, len(ref), "Lesungen,", len(types[key]), "Jahre geprüft")

with open(OUT, "w", encoding="utf-8", newline="\n") as f:
    f.write(
        "//! Wochenabschnitte je Jahrestyp (Diaspora). GENERIERT von\n"
        "//! tools/gen_parasha_table.py aus Hebcal-Jahresdaten; nicht von Hand ändern.\n"
        f"//! Geprüft an {len(years)} hebräischen Jahren ({years[0]}–{years[-1]}).\n\n"
        "/// Ein Jahrestyp (Keviyah): Wochentag von Rosch HaSchana (0 = Sonntag) und\n"
        "/// Länge des Jahres in Tagen.\n"
        "pub struct YearType {\n"
        "    pub rh_weekday: u8,\n"
        "    pub length: u16,\n"
        "    /// (Schabbat-Nummer ab dem ersten Schabbat auf oder nach Rosch HaSchana,\n"
        "    /// Index des ersten Abschnitts, ist Doppelabschnitt)\n"
        "    pub readings: &'static [(u8, u8, bool)],\n"
        "}\n\n"
        "pub static YEAR_TYPES: &[YearType] = &[\n" + "\n".join(lines) + "\n];\n"
    )
print("geschrieben:", OUT, len(types), "Typen")
