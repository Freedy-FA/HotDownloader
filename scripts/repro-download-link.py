#!/usr/bin/env python3
"""Reproduce QQ Music download-link failure for 320kmp3 / M800.

This is the feedback loop for: 歌曲下载全部失败，界面显示「网络错误，请稍后重试」。

Exit 0 when the current platform still matches the diagnosed shape:
  - CgiGetVkey for M800*.mp3 returns 104003 / empty purl
  - GetEVkey for F0M0*.mflac returns a CDN URL that serves HTTP 206

Exit 2 if the encrypted fallback itself is dead (our planned fix would not help).
"""

from __future__ import annotations

import json
import ssl
import sys
import time
import urllib.error
import urllib.request

UA = "HotDownloader/1.0"
SEARCH_URL = "https://u.y.qq.com/cgi-bin/musicu.fcg"
VKEY_URL = "https://u.y.qq.com/cgi-bin/musicu.fcg"
EKEY_URL = "https://ut.y.qq.com/cgi-bin/musicu.fcg"
CDN_HOST = "https://wx.music.tc.qq.com/"


def post(url: str, body: dict) -> dict:
    data = json.dumps(body, ensure_ascii=False).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=data,
        headers={
            "Content-Type": "application/json; charset=utf-8",
            "User-Agent": UA,
        },
        method="POST",
    )
    ctx = ssl.create_default_context()
    with urllib.request.urlopen(req, timeout=30, context=ctx) as resp:
        return json.loads(resp.read().decode("utf-8", "replace"))


def search_song(keyword: str) -> tuple[str, str, str]:
    searchid = str(int(time.time() * 1000))
    body = {
        "comm": {
            "ct": "11",
            "cv": "14090508",
            "v": "14090508",
            "tmeAppID": "qqmusic",
            "phonetype": "EBG-AN10",
            "deviceScore": "553.47",
            "devicelevel": "50",
            "newdevicelevel": "20",
            "rom": "HuaWei/EMOTION/EmotionUI_14.2.0",
            "os_ver": "12",
            "OpenUDID": "0",
            "OpenUDID2": "0",
            "QIMEI36": "0",
            "udid": "0",
            "chid": "0",
            "aid": "0",
            "oaid": "0",
            "taid": "0",
            "tid": "0",
            "wid": "0",
            "uid": "0",
            "sid": "0",
            "modeSwitch": "6",
            "teenMode": "0",
            "ui_mode": "2",
            "nettype": "1020",
            "v4ip": "",
        },
        "req": {
            "module": "music.search.SearchCgiService",
            "method": "DoSearchForQQMusicMobile",
            "param": {
                "search_type": 0,
                "searchid": searchid,
                "query": keyword,
                "page_num": 1,
                "num_per_page": 5,
                "highlight": 0,
                "nqc_flag": 0,
                "multi_zhida": 0,
                "cat": 2,
                "grp": 1,
                "sin": 0,
                "sem": 0,
            },
        },
    }
    data = post(SEARCH_URL, body)
    songs = data["req"]["data"]["body"]["item_song"]
    if not songs:
        raise SystemExit("search returned no songs")
    song = songs[0]
    mid = song["mid"]
    media = song["file"]["media_mid"]
    name = song.get("name") or song.get("title") or mid
    return name, mid, media


def cgi_get_vkey(song_id: str, filename: str) -> dict:
    body = {
        "comm": {"ct": 24, "cv": 0, "tmeAppID": "qqmusic", "format": "json"},
        "req_0": {
            "module": "vkey.GetVkeyServer",
            "method": "CgiGetVkey",
            "param": {
                "guid": "10000",
                "filename": [filename],
                "songmid": [song_id],
                "songtype": [0],
            },
        },
    }
    data = post(VKEY_URL, body)
    items = ((data.get("req_0") or {}).get("data") or {}).get("midurlinfo") or [{}]
    return items[0] if items else {}


def get_evkey(song_id: str, filename: str) -> tuple[str, str]:
    body = {
        "comm": {"ct": "19", "cv": "0", "guid": "", "tmeAppID": "qqmusic", "qq": "0"},
        "music.vkey.GetEVkey.CgiGetHotVkey": {
            "module": "music.vkey.GetEVkey",
            "method": "CgiGetHotVkey",
            "param": {"filename": [filename], "songmid": [song_id]},
        },
        "music.vkey.GetEVkey.GetEkey": {
            "module": "music.vkey.GetEVkey",
            "method": "GetEkey",
            "param": {"finfo": [{"filename": filename, "mid": song_id}]},
        },
    }
    data = post(EKEY_URL, body)
    urls = (
        ((data.get("music.vkey.GetEVkey.CgiGetHotVkey") or {}).get("data") or {}).get(
            "urls"
        )
        or [{}]
    )
    purl = (urls[0] or {}).get("purl") or ""
    ekeyinfo = (
        ((data.get("music.vkey.GetEVkey.GetEkey") or {}).get("data") or {}).get(
            "ekeyinfo"
        )
        or [{}]
    )
    ekey = (ekeyinfo[0] or {}).get("ekey") or ""
    return purl, ekey


def cdn_range_ok(purl: str) -> tuple[bool, str]:
    url = CDN_HOST + purl
    req = urllib.request.Request(
        url,
        method="GET",
        headers={
            "User-Agent": UA,
            "Referer": "https://y.qq.com",
            "Range": "bytes=0-255",
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=20) as resp:
            return resp.status == 206, f"status={resp.status}"
    except urllib.error.HTTPError as e:
        return False, f"HTTP {e.code}"
    except Exception as e:  # noqa: BLE001 - repro script should surface any CDN failure
        return False, f"{type(e).__name__}: {e}"


def main() -> int:
    keyword = sys.argv[1] if len(sys.argv) > 1 else "冻结"
    name, mid, media = search_song(keyword)
    mp3 = f"M800{media}.mp3"
    flac = f"F0M0{media}.mflac"

    print(f"song={name!r} mid={mid} media={media}")

    item = cgi_get_vkey(mid, mp3)
    result = item.get("result")
    purl_empty = not bool(item.get("purl"))
    print(f"CgiGetVkey {mp3}: result={result} purl_empty={purl_empty}")

    if not purl_empty and result in (0, None):
        print("NOTE: plain M800 vkey unexpectedly succeeded; platform may have changed")

    purl, ekey = get_evkey(mid, flac)
    print(f"GetEVkey {flac}: purl={bool(purl)} ekey_len={len(ekey)}")
    if not purl or not ekey:
        print("FAIL: encrypted fallback did not return purl+ekey")
        return 2

    ok, detail = cdn_range_ok(purl)
    print(f"CDN range {flac}: {detail}")
    if not ok:
        print("FAIL: encrypted fallback CDN is not downloadable")
        return 2

    if purl_empty or result == 104003:
        print("PASS: 320kmp3 vkey is blocked; encrypted flac fallback is downloadable")
        return 0

    print("PASS: encrypted fallback works (plain vkey also returned a purl)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
