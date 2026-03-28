import json
import re
import urllib.request
from pathlib import Path

base = 'http://127.0.0.1:4200'
text = Path('/Users/michaelgraham/Development/openfang/agents/assistant/agent.toml').read_text()
m = re.search(r'system_prompt = """(.*)"""\n\n\[\[fallback_models\]\]', text, re.S)
if not m:
    raise SystemExit('system_prompt not found')
prompt = m.group(1)
agents = json.load(urllib.request.urlopen(base + '/api/agents', timeout=10))
assistant = next(a for a in agents if a['name'] == 'assistant')
reader = next(a for a in agents if a['name'] == 'obsidian-vault-reader')
req = urllib.request.Request(
    base + '/api/agents/{}/config'.format(assistant['id']),
    data=json.dumps({'system_prompt': prompt}).encode(),
    headers={'Content-Type': 'application/json'},
    method='PATCH',
)
with urllib.request.urlopen(req, timeout=20) as r:
    print('patch', r.read().decode())
for agent in (assistant, reader):
    req = urllib.request.Request(
        base + '/api/agents/{}/session/reset'.format(agent['id']),
        data=b'{}',
        headers={'Content-Type': 'application/json'},
    )
    with urllib.request.urlopen(req, timeout=20) as r:
        print('reset', agent['name'], r.read().decode())
