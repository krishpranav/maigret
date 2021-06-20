#!/usr/bin/env python3
# generate sites from md 

# imports
from typing import Dict
import json

site_dict: Dict[str, Dict] = json.loads(open("data.json", "r", encoding="utf-8").read())