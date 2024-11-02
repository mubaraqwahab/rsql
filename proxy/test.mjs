import dotenv from "dotenv"
import path from "node:path"

dotenv.config({ path: path.resolve(import.meta.dirname, "../.env") })

const res = await fetch("http://localhost:9876", {
  method: 'post',
  body: JSON.stringify({
    url: process.env.DATABASE_URL,
    query: `select * from terms`,
  }),
  headers: {
    'Content-Type': 'application/json'
  }
})

const json = await res.json()

console.log('resp json', json)