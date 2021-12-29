import regeneratorRuntime from "regenerator-runtime"
const { expect } = require('chai')
const axios = require('axios').default
const api = 'http://127.0.0.1:8080/api'

const headers = {
    'Content-Type': 'application/json'
}

function json(data) {
    return JSON.stringify(data)
}

describe('/api/', function () {
    describe('/msg', function () {
        beforeEach(async function () {
            await axios(`${api}/group`, { method: 'DELETE', params: { priority: 1 } })            
        })
        describe('GET', function () {
            it('should return inserted uuid with no paramaters', async function () {
                let uuid = await (await axios.post(`${api}/msg`, { "priority": 1, "msg": "foo" })).data.uuid
                let getResponse = await axios.get(`${api}/msg`)
                expect(getResponse.data.uuid).to.eql(uuid)
                // console.log(res)
            })
            it('should return inserted uuid with uuid parameter', async function () {
                let uuid1 = await (await axios.post(`${api}/msg`, { "priority": 1, "msg": "foo" })).data.uuid
                let uuid2 = await (await axios.post(`${api}/msg`, { "priority": 2, "msg": "foo" })).data.uuid
                let getResponse = await axios.get(`${api}/msg?uuid=${uuid1}`)
                expect(getResponse.data.uuid).to.eql(uuid1)
                // console.log(res)
            })
            it('should return inserted uuid with priority parameter', async function () {
                let uuid1 = await (await axios.post(`${api}/msg`, { "priority": 1, "msg": "foo" })).data.uuid
                let uuid2 = await (await axios.post(`${api}/msg`, { "priority": 2, "msg": "foo" })).data.uuid
                let getResponse = await axios.get(`${api}/msg?priority=1`)
                expect(getResponse.data.uuid).to.eql(uuid1)
                // console.log(res)
            })
        })
    })
})