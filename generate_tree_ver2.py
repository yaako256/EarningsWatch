import sys
from pathlib import Path

# ツリー描画用の記号
SPACE = '    '
BRANCH = '├── '
LAST_BRANCH = '└── '
VERTICAL = '│   '

def count_items_recursive(directory, max_depth, exclude_dirs, current_depth=1):
    """指定された深さまで全アイテムを再帰的にカウント（除外ディレクトリを考慮）"""
    if current_depth > max_depth:
        return 0
    try:
        count = 0
        for p in directory.iterdir():
            if p.is_dir() and p.name in exclude_dirs:
                continue
            count += 1
            if p.is_dir():
                count += count_items_recursive(p, max_depth, exclude_dirs, current_depth + 1)
        return count
    except PermissionError:
        return 0

def draw_tree(directory, max_depth, exclude_dirs, current_depth=1, prefix=""):
    """指定された深さまでツリーを描画（除外ディレクトリを考慮）"""
    if current_depth > max_depth:
        return

    try:
        all_paths = list(directory.iterdir())
    except PermissionError:
        return

    # 除外対象のディレクトリをフィルタリング
    filtered_paths = [
        p for p in all_paths 
        if not (p.is_dir() and p.name in exclude_dirs)
    ]

    files = sorted([p for p in filtered_paths if p.is_file()], key=lambda p: p.name.lower())
    
    dirs_with_counts = []
    for d in [p for p in filtered_paths if p.is_dir()]:
        c = count_items_recursive(d, max_depth, exclude_dirs, current_depth + 1)
        dirs_with_counts.append((d, c))
    
    dirs_with_counts.sort(key=lambda x: (x[1], x[0].name.lower()))
    dirs = [d[0] for d in dirs_with_counts]
    
    paths = files + dirs
    count = len(paths)

    for i, path in enumerate(paths):
        is_last = (i == count - 1)
        connector = LAST_BRANCH if is_last else BRANCH
        suffix = "/" if path.is_dir() else ""
        
        print(f"{prefix}{connector}{path.name}{suffix}")
        
        if path.is_dir():
            new_prefix = prefix + (SPACE if is_last else VERTICAL)
            draw_tree(path, max_depth, exclude_dirs, current_depth + 1, new_prefix)

def main():
    # 1. まずデフォルト値を設定（引数がゼロの時はこれで動く）
    target_path = Path('.')
    max_depth = float('inf')  # デフォルトは無制限
    exclude_list = []

    # 2. 第1引数があれば「対象パス」を上書き
    if len(sys.argv) > 1:
        target_path = Path(sys.argv[1])

    # 3. 第2引数があれば「深さ」を上書き
    if len(sys.argv) > 2:
        try:
            max_depth = int(sys.argv[2])
        except ValueError:
            # 万が一、数値以外が入れられた場合は無制限(inf)のまま進める
            pass

    # 4. 第3引数以降があれば「除外リスト」にする
    if len(sys.argv) > 3:
        exclude_list = sys.argv[3:]

    # パスの存在チェック
    if not target_path.exists():
        print(f"Error: {target_path} は存在しません。", file=sys.stderr)
        return

    # 除外リストをセットに変換
    exclude_dirs = set(exclude_list)

    print(f"{target_path.resolve().name}/")
    draw_tree(target_path, max_depth, exclude_dirs)

if __name__ == "__main__":
    main()
    