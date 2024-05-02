module.exports = {
    title: "Fast Search",
    description: "Fast Search is private search engine, alternative to google, which is build using rust",
    themeConfig: {
        nav: [
            { text: "Home", link: "/" },
            { text: "Github", link: "https://github.com/Himasnhu-AT/FastSearch.git" },

        ],
        sidebar: [
            {
                text: "Getting Started",
                items: [
                    { text: "Home", link: "/" },
                    { text: "Get Started", link: "/readme" },
                    { text: "TO DO", link: "/todo" },
                    { text: "LICESE", link: "/license" },
                    { text: "Changelog", link: "/changelog" },
                    { text: "Contributing", link: "/contributing" }
                ]
            },
            {
                text: "Code Explanation",
                items: [
                    { text: "Code Structure", link: "/code-structure" },
                    { text: "Code Explanation", link: "/code-explanation" }

                ]
            },
            {
                text: "Research Used",
                items: [
                    { text: "TF-IDf", link: "/tf_idf" }
                ]
            }
        ]
    }
};