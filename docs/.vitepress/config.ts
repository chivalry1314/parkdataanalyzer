import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'PARKINGSYSLOG',
  description: '停车收费记录解析工具操作文档',
  base: '/parkdataanalyzer/',
  lang: 'zh-CN',
  lastUpdated: true,
  themeConfig: {
    nav: [
      { text: '首页', link: '/' },
      { text: '操作指南', link: '/guide/' },
      { text: '常见问题', link: '/guide/faq' },
    ],
    sidebar: {
      '/guide/': [
        {
          text: '开始',
          items: [
            { text: '文档概述', link: '/guide/' },
            { text: '安装与启动', link: '/guide/install' },
          ],
        },
        {
          text: '功能模块',
          items: [
            { text: '记录列表', link: '/guide/records' },
            { text: '车辆收费', link: '/guide/vehicle' },
            { text: '统计分析', link: '/guide/stats' },
            { text: '设置', link: '/guide/settings' },
          ],
        },
        {
          text: '其他',
          items: [
            { text: '常见问题', link: '/guide/faq' },
          ],
        },
      ],
    },
    socialLinks: [
      { icon: 'github', link: 'https://github.com/chivalry1314/parkdataanalyzer' },
    ],
    footer: {
      message: 'PARKINGSYSLOG 在线操作文档',
      copyright: 'Copyright © 2026',
    },
    search: {
      provider: 'local',
    },
  },
})
