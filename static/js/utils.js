async function send_req(form, data, callback) {
  return fetch(form.action, {
    method: "POST",
    body: JSON.stringify(data),
    headers: {
      "Content-type": "application/json; charset=UTF-8"
    },
  })
    .then((response) => {
      if (!response.ok || response.status.toString()[0] != "2") {
        throw Error("Status: " + response.status);
      }
      return response.json();
    })
    .then((json) => {
      callback(json, form, data)
    })
    .catch((err) => {
      console.error("Could not submit form: " + err);
      const err_div = form.querySelector("#error");
      if (!err_div) return;
      err_div.hidden = false;
      err_div.innerHTML = err;
    })
}

function show_form_errors(form, errors) {
  errors.forEach((err) => {
    const name = err.name;
    const desc = err.description;

    const inp = form.querySelector(`[name="${name}"]`).parentElement;
    inp.classList.add('outlined');
    inp.classList.add('error');

    if (desc !== "") {
      const error_text = document.createElement('span');
      error_text.id = 'error-text';
      error_text.classList.add('supporting-text')
      error_text.innerHTML = desc;

      inp.appendChild(error_text)
    }

    const error_icon = document.createElement('div');
    error_icon.id = 'error-icon';
    error_icon.classList.add('suffix')
    error_icon.innerHTML = '<i class="material-icons">error</i>';

    inp.appendChild(error_icon)
  });
}

function reset_form(form) {
  form.querySelectorAll('.input-field').forEach((elem) => {
    elem.querySelectorAll('textarea').forEach((x) => x.value = '');
    elem.querySelectorAll('input').forEach((x) => x.value = '');

    elem.classList.remove('outlined');
    elem.classList.remove('error');
    const icon = elem.querySelector('#error-icon');
    if (icon) icon.remove();
    const text = elem.querySelector('#error-text');
    if (text) text.remove();
  });

  const err_div = form.querySelector("#error");
  if (!err_div) return;
  err_div.hidden = true;
}

function init_form(orig_form, callback, validate) {
  const handleSubmit = (event) => {
    event.preventDefault();

    const form = event.target;
    const btn = form.querySelector("input[type='submit']");
    btn.disabled = true;

    let data = get_form_obj(form);
    if (validate) {
      data = validate(form, data);
    }
    if (!data) return;

    send_req(form, data, callback)
      .finally(() => btn.disabled = false);
  }

  orig_form.addEventListener("submit", handleSubmit);
}

function get_form_obj(form) {
  return Object.fromEntries(new FormData(form));
}

function copy_text_in(id) {
  var url_input = document.getElementById(id);

  // Select the text field
  url_input.select();
  url_input.setSelectionRange(0, 99999); // For mobile devices

   // Copy the text inside the text field
  navigator.clipboard.writeText(url_input.value);
}


document.addEventListener('DOMContentLoaded', function() {
  M.Modal.init(document.querySelectorAll('.modal'), {
    startingTop: '10%',
    endingTop: '20%',
  });

  M.Tooltip.init(document.querySelectorAll('.tooltipped'), {
    enterDelay: 10,
  });

  M.FormSelect.init(document.querySelectorAll('select'), {});
});
