export interface IPicturesResponse {
    url: string
    preview_url: string
    name: string
    saved: boolean
}

export class ProductService {
    async getImages(): Promise<IPicturesResponse[]> {
        return [
            {
                url: 'https://images.hdqwalls.com/download/pubg-game-4k-qx-1920x1080.jpg',
                preview_url: 'https://images.hdqwalls.com/download/pubg-game-4k-qx-640x480.jpg',
                name: 'Pubg Game 4k',
                saved: true
            },
            {
                url: 'https://images.hdqwalls.com/download/marvels-spider-man-pc-4k-jf-1920x1080.jpg',
                preview_url: 'https://images.hdqwalls.com/download/marvels-spider-man-pc-4k-jf-640x480.jpg',
                name: 'Spider man',
                saved: false
            },
            {
                url: 'https://images.hdqwalls.com/download/ciri-cyberpunk-2077-tv-1920x1080.jpg',
                preview_url: 'https://images.hdqwalls.com/download/ciri-cyberpunk-2077-tv-640x480.jpg',
                name: 'Pubg Game 4k',
                saved: true
            },
            {
                url: 'https://images.hdqwalls.com/download/pubg-game-4k-qx-1920x1080.jpg',
                preview_url: 'https://images.hdqwalls.com/download/pubg-game-4k-qx-640x480.jpg',
                name: 'Pubg Game 4k',
                saved: true
            },
            {
                url: 'https://images.hdqwalls.com/download/marvels-spider-man-pc-4k-jf-1920x1080.jpg',
                preview_url: 'https://images.hdqwalls.com/download/marvels-spider-man-pc-4k-jf-640x480.jpg',
                name: 'Spider man',
                saved: false
            },
            {
                url: 'https://images.hdqwalls.com/download/ciri-cyberpunk-2077-tv-1920x1080.jpg',
                preview_url: 'https://images.hdqwalls.com/download/ciri-cyberpunk-2077-tv-640x480.jpg',
                name: 'Pubg Game 4k',
                saved: true
            }
        ]
    }

}