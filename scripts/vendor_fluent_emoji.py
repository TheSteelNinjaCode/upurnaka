"""Vendor the channel picker's Fluent 3D artwork and metadata, pinned to upstream.

Run deliberately with Python; application builds never download these assets.
Images are copied unchanged. Microsoft copyright and MIT license accompany them.
"""
import concurrent.futures
import json
from pathlib import Path
import time
import urllib.parse
import urllib.request

REVISION = "1ffb34c752ecf5d402f04cfb4b392c77f57c54bc"
ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / "public/images/fluent-emoji"
GROUPS = {
    "smileys": "grinning face|grinning face with big eyes|grinning face with smiling eyes|beaming face with smiling eyes|grinning squinting face|grinning face with sweat|rolling on the floor laughing|face with tears of joy|slightly smiling face|upside-down face|winking face|smiling face with smiling eyes|smiling face with halo|smiling face with hearts|smiling face with heart-eyes|star-struck|face blowing a kiss|kissing face|smiling face|face savoring food|face with tongue|winking face with tongue|zany face|hugging face|face with hand over mouth|shushing face|thinking face|saluting face|zipper-mouth face|face with raised eyebrow|neutral face|expressionless face|face without mouth|smirking face|unamused face|face with rolling eyes|grimacing face|lying face|relieved face|pensive face|sleepy face|drooling face|sleeping face|face with medical mask|face with thermometer|face with head-bandage|nauseated face|sneezing face|hot face|cold face|woozy face|face with crossed-out eyes|exploding head|cowboy hat face|partying face|smiling face with sunglasses|nerd face|face with monocle|confused face|worried face|slightly frowning face|frowning face|face with open mouth|hushed face|astonished face|flushed face|pleading face|frowning face with open mouth|anguished face|fearful face|anxious face with sweat|sad but relieved face|crying face|loudly crying face|face screaming in fear|confounded face|persevering face|disappointed face|downcast face with sweat|weary face|tired face|yawning face|face with steam from nose|pouting face|angry face|smiling face with horns|skull|pile of poo|clown face|ghost|alien|robot|grinning cat|smiling cat with heart-eyes|see-no-evil monkey|hear-no-evil monkey|speak-no-evil monkey",
    "gestures": "waving hand|raised back of hand|hand with fingers splayed|raised hand|vulcan salute|ok hand|pinched fingers|pinching hand|victory hand|crossed fingers|love-you gesture|sign of the horns|call me hand|backhand index pointing left|backhand index pointing right|backhand index pointing up|backhand index pointing down|index pointing up|thumbs up|thumbs down|raised fist|oncoming fist|left-facing fist|right-facing fist|clapping hands|raising hands|open hands|palms up together|handshake|folded hands|writing hand|nail polish|flexed biceps|eyes",
    "people": "baby|child|boy|girl|person|man|woman|older person|old man|old woman|person with crown|prince|princess|person wearing turban|person with skullcap|person with headscarf|police officer|detective|guard|ninja|construction worker|person in tuxedo|person with veil|pregnant woman|breast-feeding|person running|person walking|person standing|person kneeling|person in lotus position|person taking bath|people hugging|bust in silhouette|busts in silhouette|family|student|teacher|technologist|astronaut|health worker|scientist|artist|firefighter|cook|mechanic",
    "animals": "dog face|cat face|mouse face|hamster|rabbit face|fox|bear|panda|koala|tiger face|lion|cow face|pig face|frog|monkey face|chicken|penguin|bird|baby chick|duck|eagle|owl|bat|wolf|horse face|unicorn|honeybee|butterfly|snail|lady beetle|turtle|snake|octopus|tropical fish|dolphin|whale|shark|elephant|giraffe|deer|hedgehog|paw prints|seedling|herb|four leaf clover|deciduous tree|evergreen tree|palm tree|cactus|sunflower|rose|hibiscus|blossom|cherry blossom|tulip|mushroom",
    "food": "red apple|green apple|pear|tangerine|lemon|banana|watermelon|grapes|strawberry|blueberries|cherries|peach|pineapple|kiwi fruit|avocado|broccoli|carrot|ear of corn|hot pepper|cucumber|croissant|bread|pretzel|cheese wedge|hamburger|french fries|pizza|hot dog|sandwich|taco|burrito|cooking|shallow pan of food|sushi|cooked rice|spaghetti|dumpling|ice cream|doughnut|cookie|birthday cake|shortcake|cupcake|chocolate bar|candy|popcorn|hot beverage|teacup without handle|bubble tea|clinking glasses",
    "travel": "globe showing europe-africa|globe showing americas|globe showing asia-australia|world map|mountain|snow-capped mountain|camping|beach with umbrella|desert island|national park|stadium|classical building|house|house with garden|office building|school|hospital|bank|hotel|factory|castle|fountain|ferris wheel|roller coaster|sunrise|sunset|cityscape|bridge at night|automobile|taxi|bus|oncoming bus|racing car|bicycle|motor scooter|motorcycle|locomotive|high-speed train|airplane|helicopter|rocket|sailboat|ship|canoe|rainbow|sun|cloud|cloud with rain|snowflake|fire|star|crescent moon",
    "objects": "speech balloon|megaphone|loudspeaker|bell|light bulb|flashlight|candle|laptop|desktop computer|keyboard|computer mouse|mobile phone|telephone|camera|video camera|television|radio|microphone|headphone|books|open book|closed book|blue book|green book|notebook|ledger|page facing up|memo|pencil|fountain pen|paintbrush|crayon|scissors|paperclip|pushpin|round pushpin|straight ruler|triangular ruler|file folder|open file folder|card file box|calendar|spiral calendar|clipboard|bar chart|chart increasing|chart decreasing|package|envelope|incoming envelope|ballot box with ballot|locked|unlocked|key|hammer|wrench|gear|toolbox|magnet|test tube|microscope|telescope|pill|stethoscope|graduation cap|briefcase|shopping bags|gem stone|t-shirt|crown|glasses|sunglasses",
    "activities": "soccer ball|basketball|american football|baseball|tennis|volleyball|rugby football|flying disc|bowling|cricket game|ping pong|badminton|ice hockey|goal net|golf flag|fishing pole|boxing glove|martial arts uniform|running shirt|skis|trophy|1st place medal|2nd place medal|3rd place medal|sports medal|military medal|video game|joystick|game die|jigsaw|chess pawn|bullseye|performing arts|artist palette|musical note|musical notes|guitar|musical keyboard|violin|drum|saxophone|trumpet|party popper|confetti ball|balloon|wrapped gift|ribbon|ticket|clapper board",
    "symbols": "red heart|orange heart|yellow heart|green heart|blue heart|purple heart|black heart|white heart|brown heart|broken heart|sparkling heart|two hearts|heart on fire|hundred points|anger symbol|collision|sweat droplets|dizzy|check mark|check mark button|cross mark|plus|minus|multiply|divide|infinity|question mark|red question mark|red exclamation mark|exclamation mark|double exclamation mark|warning|prohibited|no entry|recycling symbol|sparkles|glowing star|fireworks|soon arrow|right arrow|left arrow|up arrow|down arrow|counterclockwise arrows button|play button|pause button|stop button|record button|musical score|keycap number sign|information|registered|copyright",
}


def get(path):
    url = f"https://raw.githubusercontent.com/microsoft/fluentui-emoji/{REVISION}/" + urllib.parse.quote(path, safe="/")
    for attempt in range(3):
        try:
            with urllib.request.urlopen(url, timeout=45) as response:
                return response.read()
        except Exception:
            if attempt == 2:
                raise
            time.sleep(attempt + 1)


def main():
    cache = ROOT / ".rahti/fluent-emoji-tree.json"
    if cache.exists():
        tree = json.loads(cache.read_text(encoding="utf-8"))
        if tree["sha"] != REVISION:
            raise ValueError("The cached tree does not match the pinned revision")
    else:
        request = urllib.request.Request(f"https://api.github.com/repos/microsoft/fluentui-emoji/git/trees/{REVISION}?recursive=1", headers={"User-Agent": "upurnaka-assets"})
        with urllib.request.urlopen(request, timeout=60) as response:
            tree = json.load(response)
    paths = {item["path"] for item in tree["tree"]}
    folders = {path.split("/")[1].lower(): path.rsplit("/", 1)[0] for path in paths if path.endswith("/metadata.json")}
    DEST.mkdir(parents=True, exist_ok=True)
    tasks, missing = [], []
    for category, names in GROUPS.items():
        for name in names.split("|"):
            folder = folders.get(name)
            if folder:
                choices = sorted(path for path in paths if path.startswith(folder + "/") and "/3D/" in path and path.endswith(".png") and (path.startswith(folder + "/3D/") or path.startswith(folder + "/Default/3D/")))
                if choices:
                    tasks.append((category, folder, choices[0]))
                else:
                    missing.append(name)
            else:
                missing.append(name)

    def vendor(task):
        category, folder, asset = task
        metadata = json.loads(get(folder + "/metadata.json"))
        glyph = metadata["glyph"]
        filename = "-".join(f"{ord(character):x}" for character in glyph) + ".png"
        destination = DEST / filename
        if not destination.exists():
            destination.write_bytes(get(asset))
        return {"glyph": glyph, "name": metadata["cldr"], "keywords": metadata.get("keywords", []), "category": category, "image": "/images/fluent-emoji/" + filename}

    with concurrent.futures.ThreadPoolExecutor(max_workers=12) as executor:
        catalog = list(executor.map(vendor, tasks))
    seen = set()
    catalog = [entry for entry in catalog if not (entry["glyph"] in seen or seen.add(entry["glyph"]))]
    (DEST / "catalog.json").write_text(json.dumps(catalog, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    (DEST / "LICENSE").write_bytes(get("LICENSE"))
    (DEST / "SOURCE.md").write_text(f"# Fluent Emoji\n\nOriginal, unmodified Microsoft Fluent Emoji 3D PNG files.\n\nSource: https://github.com/microsoft/fluentui-emoji\nRevision: `{REVISION}`\nLicense: MIT; see LICENSE.\n\nRebuild this curated catalog with `python scripts/vendor_fluent_emoji.py`.\n", encoding="utf-8")
    print(json.dumps({"count": len(catalog), "bytes": sum(path.stat().st_size for path in DEST.glob("*.png")), "unavailableNames": missing}, ensure_ascii=True))


if __name__ == "__main__":
    main()
