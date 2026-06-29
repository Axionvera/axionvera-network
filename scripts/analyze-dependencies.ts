import * as fs from 'fs';
import * as path from 'path';

/**
 * Interface representing a contract and its dependencies.
 */
interface ContractDependency {
  name: string;
  path: string;
  dependencies: string[];
}

/**
 * Analyzes contract dependencies by parsing Cargo.toml files.
 */
class DependencyAnalyzer {
  private contractsPath: string;

  constructor(contractsPath: string) {
    this.contractsPath = contractsPath;
  }

  /**
   * Main execution method.
   */
  public async analyze() {
    console.log(`Scanning contracts in: ${this.contractsPath}`);
    const contracts = this.findContracts();
    console.log(`Found ${contracts.length} contracts.`);

    const graph = this.buildDependencyGraph(contracts);
    const cycles = this.detectCycles(graph);

    this.generateReport(graph, cycles);
    this.generateMermaidDiagram(graph);
  }

  private findContracts(): string[] {
    const contracts: string[] = [];
    const dirs = fs.readdirSync(this.contractsPath);
    for (const dir of dirs) {
      const fullPath = path.join(this.contractsPath, dir);
      if (fs.statSync(fullPath).isDirectory()) {
        const cargoPath = path.join(fullPath, 'Cargo.toml');
        if (fs.existsSync(cargoPath)) {
          contracts.push(dir);
        }
      }
    }
    return contracts;
  }

  private buildDependencyGraph(contractDirs: string[]): Map<string, string[]> {
    const graph = new Map<string, string[]>();

    for (const dir of contractDirs) {
      const cargoPath = path.join(this.contractsPath, dir, 'Cargo.toml');
      const content = fs.readFileSync(cargoPath, 'utf-8');

      const deps: string[] = [];
      // Simple regex to find local path dependencies in Cargo.toml
      // Matches: axionvera-auth = { path = "../auth" }
      const re = /([a-zA-Z0-9_-]+)\s*=\s*{\s*path\s*=\s*"(\.\.\/([^"]+))"/g;
      let match;
      while ((match = re.exec(content)) !== null) {
        const depName = match[3];
        if (contractDirs.includes(depName)) {
          deps.push(depName);
        }
      }
      graph.set(dir, deps);
    }

    return graph;
  }

  private detectCycles(graph: Map<string, string[]>): string[][] {
    const cycles: string[][] = [];
    const visited = new Set<string>();
    const recStack = new Set<string>();
    const path: string[] = [];

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

    return cycles;
  }

  private generateReport(graph: Map<string, string[]>, cycles: string[][]) {
    let report = '# Protocol Contract Dependency Report\n\n';

    if (cycles.length > 0) {
      report += '## ⚠️ Circular Dependencies Detected!\n';
      cycles.forEach((cycle, i) => {
        report += `${i + 1}. ${cycle.join(' -> ')}\n`;
      });
      report += '\n';
    } else {
      report += '✅ No circular dependencies detected.\n\n';
    }

    report += '## Dependency List\n';
    for (const [contract, deps] of Array.from(graph.entries()).sort()) {
      if (deps.length > 0) {
        report += `- **${contract}** depends on: ${deps.join(', ')}\n`;
      } else {
        report += `- **${contract}** has no internal dependencies.\n`;
      }
    }

    const reportPath = path.join(process.cwd(), 'docs/architecture/dependency-report.md');
    const docsDir = path.dirname(reportPath);
    if (!fs.existsSync(docsDir)) {
      fs.mkdirSync(docsDir, { recursive: true });
    }
    fs.writeFileSync(reportPath, report);
    console.log(`Report generated at: ${reportPath}`);
  }

  private generateMermaidDiagram(graph: Map<string, string[]>) {
    let mermaid = '# Protocol Architecture Diagram\n\n';
    mermaid += '```mermaid\ngraph TD\n';

    for (const [contract, deps] of graph.entries()) {
      if (deps.length === 0 && Array.from(graph.values()).every(d => !d.includes(contract))) {
         mermaid += `    ${contract.replace(/-/g, '_')}[${contract}]\n`;
      }
      for (const dep of deps) {
        mermaid += `    ${contract.replace(/-/g, '_')}[${contract}] --> ${dep.replace(/-/g, '_')}[${dep}]\n`;
      }
    }

    mermaid += '```\n';

    const diagramPath = path.join(process.cwd(), 'docs/architecture/dependency-graph.md');
    fs.writeFileSync(diagramPath, mermaid);
    console.log(`Diagram generated at: ${diagramPath}`);
  }
}

const analyzer = new DependencyAnalyzer(path.join(process.cwd(), 'contracts'));
analyzer.analyze().catch(err => {
  console.error('Analysis failed:', err);
  process.exit(1);
});
