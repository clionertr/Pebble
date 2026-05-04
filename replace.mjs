import fs from 'fs';
import path from 'path';

function walk(dir) {
  let results = [];
  const list = fs.readdirSync(dir);
  list.forEach(function(file) {
    file = dir + '/' + file;
    const stat = fs.statSync(file);
    if (stat && stat.isDirectory()) { 
      results = results.concat(walk(file));
    } else { 
      if (file.endsWith('.ts') || file.endsWith('.tsx')) {
        results.push(file);
      }
    }
  });
  return results;
}

const files = walk('src');
for (const file of files) {
  let content = fs.readFileSync(file, 'utf8');
  let changed = false;
  
  if (content.includes('@tauri-apps/api')) {
    // determine relative depth
    const depth = file.split('/').length - 2; // src/app/Layout.tsx -> depth 1
    const prefix = depth === 0 ? './' : '../'.repeat(depth);
    const mockPath = prefix + 'tauri-mock';
    
    content = content.replace(/"@tauri-apps\/api\/.*?"/g, `"${mockPath}"`);
    changed = true;
  }
  
  if (changed) {
    fs.writeFileSync(file, content);
    console.log(`Updated ${file}`);
  }
}
