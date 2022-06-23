const produceButton = document.querySelector('#produce-btn')
const consumeButton = document.querySelector('#consume-btn')
const stopConsumeButton = document.querySelector('#stop-consume-btn')

const responseContainer = document.querySelector('#response')
const topicContainer = document.querySelector('#topics')
const listTopicsButton = document.querySelector('#list-btn')

const invoke = window.__TAURI__.invoke

function updateResponse(response) {
    responseContainer.innerText =
        typeof response === 'string' ? response : JSON.stringify(response)
}

function updateTopicList(response) {
    topicContainer.innerText = Object.values(response).join("\n")
}

produceButton.addEventListener('click', () => {
    invoke('send_message', {
        numOfMessages: 1
    })
        .then(updateResponse)
        .catch(updateResponse)
})

consumeButton.addEventListener('click', () => {
    invoke('consume')
        .then(updateResponse)
        .catch(updateResponse)
})


stopConsumeButton.addEventListener('click', () => {
    invoke('stop_consumer')
})

listTopicsButton.addEventListener('click', () => {
    invoke('list_topics')
        .then(updateTopicList)
        .catch(updateTopicList)
})
