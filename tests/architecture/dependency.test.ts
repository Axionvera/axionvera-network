import { describe, it, expect } from 'vitest';
import * as fs from 'fs';
import * as path from 'path';

describe('Protocol Architecture Dependencies', () => {
  const contractsPath = path.join(process.cwd(), 'contracts');
  const dirs = fs.readdirSync(contractsPath);
  const contracts = dirs.filter(dir => {
    const fullPath = path.join(contractsPath, dir);
    return fs.statSync(fullPath).isDirectory() && fs.existsSync(path.join(fullPath, 'Cargo.toml'));
  });

  const graph = new Map<string, string[]>();
  for (const dir of contracts) {
    const cargoPath = path.join(contractsPath, dir, 'Cargo.toml');
    const content = fs.readFileSync(cargoPath, 'utf-8');
    const deps: string[] = [];
    const re = /([a-zA-Z0-9_-]+)\s*=\s*{\s*path\s*=\s*"(\.\.\/([^"]+))"/g;
    let match;
    while ((match = re.exec(content)) !== null) {
      const depName = match[3];
      if (contracts.includes(depName)) {
        deps.push(depName);
      }
    }
    graph.set(dir, deps);
  }

  it('should not have circular dependencies', () => {
    const visited = new Set<string>();
    const recStack = new Set<string>();
    const path: string[] = [];
    const cycles: string[][] = [];

    const dfs = (node: string) => {
      visited.add(node);
      recStack.add(node);
      path.push(node);

      const neighbors = graph.get(node) || [];
      for (const neighbor of neighbors) {
        if (!visited.has(neighbor)) {
          dfs(neighbor);
        } else if (recStack.has(neighbor)) {
          const cycleStart = path.indexOf(neighbor);
          cycles.push([...path.slice(cycleStart), neighbor]);
        }
      }

      recStack.delete(node);
      path.pop();
    };

    for (const node of graph.keys()) {
      if (!visited.has(node)) {
        dfs(node);
      }
    }

    expect(cycles, `Circular dependencies detected: ${cycles.map(c => c.join(' -> ')).join(', ')}`).toHaveLength(0);
  });

  it('should have a generated dependency graph', () => {
    const graphPath = path.join(process.cwd(), 'docs/architecture/dependency-graph.md');
    expect(fs.existsSync(graphPath), 'Dependency graph documentation should exist').toBe(true);
  });

  it('should have a generated dependency report', () => {
    const reportPath = path.join(process.cwd(), 'docs/architecture/dependency-report.md');
    expect(fs.existsSync(reportPath), 'Dependency report documentation should exist').toBe(true);
  });
});
