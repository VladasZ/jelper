#!/usr/bin/env python3
"""Release helper script."""
import subprocess
import re
import sys

def get_next_version(bump_type):
    """Get the next version tag."""
    tag = subprocess.check_output(['git', 'describe', '--tags', '--abbrev=0'], text=True).strip()
    match = re.match(r'v(\d+)\.(\d+)\.(\d+)', tag)

    if not match:
        print(f'Failed to parse version from {tag}')
        sys.exit(1)

    major, minor, patch = match.groups()

    if bump_type == 'patch':
        new_patch = int(patch) + 1
        new_tag = f'v{major}.{minor}.{new_patch}'
    elif bump_type == 'minor':
        new_minor = int(minor) + 1
        new_tag = f'v{major}.{new_minor}.0'
    else:
        print(f'Unknown bump type: {bump_type}')
        sys.exit(1)

    subprocess.run(['git', 'tag', new_tag], check=True)
    subprocess.run(['git', 'push', 'origin', new_tag], check=True)
    print(f'Released {new_tag}')

if __name__ == '__main__':
    bump_type = sys.argv[1] if len(sys.argv) > 1 else 'patch'
    get_next_version(bump_type)
