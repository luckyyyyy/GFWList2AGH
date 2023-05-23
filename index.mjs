import Got from 'got';
import fs from 'node:fs/promises';

const local_dns = '127.0.0.1';
const remote_dns = '127.0.0.1:1053';


const cn = [
  'https://raw.githubusercontent.com/blackmatrix7/ios_rule_script/master/rule/Clash/ChinaMaxNoIP/ChinaMaxNoIP_Domain.txt',
  "https://raw.githubusercontent.com/Loyalsoldier/v2ray-rules-dat/release/apple-cn.txt",
  "https://raw.githubusercontent.com/Loyalsoldier/v2ray-rules-dat/release/direct-list.txt",
  "https://raw.githubusercontent.com/Loyalsoldier/v2ray-rules-dat/release/google-cn.txt",
  "https://raw.githubusercontent.com/blackmatrix7/ios_rule_script/master/rule/Surge/China/China_Domain.list",
]
const domains = { cn: true, lan: true };
// Promise.all并发请求 获取域名数据
const getDomain = async () => {
  const res = await Promise.all(cn.map(url => Got(url)));
  res.forEach(item => {
    const arr = item.body.split('\n');
    arr.forEach(domain => {
      if (domain && !(domain.startsWith('#'))) {
        // Extract domain from each line
        const domainMatch = domain.match(/([a-z0-9]+(-[a-z0-9]+)*\.)+[a-z]{2,}/);
        if (domainMatch) {
          const domain = domainMatch[0];
          // You can add your domain to your 'domains' object here
          domains[domain] = true;
        }
      }
    })
  })
}

await getDomain();

// console.log(Object.keys(domains))
// 删除文件夹 不报错
try {
  await fs.rmdir('./AdGuardHome', { recursive: true, });
} catch (error) {}
await fs.mkdir('./AdGuardHome', { recursive: true });
await fs.writeFile('./AdGuardHome/domains.txt',
  remote_dns + '\n' + Object.keys(domains).map((item) => {
    return `[/${item}/]${local_dns}`;
  }).join('\n')
);
await fs.writeFile('./AdGuardHome/domains-min.txt',
  remote_dns + '\n' + '[/' + Object.keys(domains).join('/') + '/]' + local_dns
);