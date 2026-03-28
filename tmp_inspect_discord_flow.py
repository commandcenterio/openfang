import json
import sqlite3
import urllib.request
from pathlib import Path

base = 'http://127.0.0.1:4200'
print('--- health ---')
print(urllib.request.urlopen(base + '/api/health', timeout=5).read().decode())

print('\n--- channels ---')
channels = json.load(urllib.request.urlopen(base + '/api/channels', timeout=10))
print(json.dumps(channels, indent=2)[:4000])

print('\n--- agents ---')
agents = json.load(urllib.request.urlopen(base + '/api/agents', timeout=10))
for a in sorted(agents, key=lambda x: x['name'].lower()):
    if a['name'] in {'assistant', 'obsidian-vault-reader', 'obsidian-vault-writer'} or 'project' in a['name'].lower():
        print(json.dumps({k: a.get(k) for k in ['id', 'name', 'state', 'ready', 'model_provider', 'model_name']}, indent=2))

print('\n--- assistant session tail ---')
assistant = next(a for a in agents if a['name'] == 'assistant')
session = json.load(urllib.request.urlopen(base + f"/api/agents/{assistant['id']}/session", timeout=10))
print('message_count', session['message_count'])
for m in session.get('messages', [])[-10:]:
    print(m.get('role'), '|', (m.get('content') or '').replace('\n', ' ')[:300])

print('\n--- sqlite recent audit entries ---')
db = sqlite3.connect(str(Path.home() / '.openfang' / 'data' / 'openfang.db'))
for row in db.execute("select seq,timestamp,agent_id,action,outcome,substr(detail,1,240) from audit_entries order by seq desc limit 30"):
    print(row)

db.close()
